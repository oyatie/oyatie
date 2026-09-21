//! The DNS answers message authentication is allowed to see.
//!
//! `mail-auth` consults a `ResolverCache` *before* its own resolver, so this
//! one type is the whole seam: a conformance run is hermetic without a second
//! code path through the protocol.
use mail_auth::{
    DnsError, DnssecStatus, Error, MX, RecordSet, ResolverCache, Txt,
    hickory_resolver::proto::op::ResponseCode,
};
use std::{
    borrow::Borrow,
    collections::HashMap,
    hash::Hash,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    sync::Mutex,
    time::Instant,
};

struct Table<K, V>(Mutex<HashMap<K, V>>);

// Derived `Default` would demand `V: Default`, and a DNS record has no
// default: an empty table is the only thing "no records" can mean.
impl<K, V> Default for Table<K, V> {
    fn default() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
}

impl<K: Eq + Hash, V: Clone> Table<K, V> {
    fn get<Q>(&self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.0.lock().expect("dns table").get(key).cloned()
    }

    fn remove<Q>(&self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.0.lock().expect("dns table").remove(key)
    }

    fn put(&self, key: K, value: V) {
        self.0.lock().expect("dns table").insert(key, value);
    }
}

/// Answers `mail-auth` reads before it resolves. Open, every lookup misses and
/// the resolver behind it answers; sealed, see `resolver_cache!`.
#[derive(Default)]
pub struct MailDns {
    txt: Table<Box<str>, Txt>,
    mx: Table<Box<str>, RecordSet<MX>>,
    ipv4: Table<Box<str>, RecordSet<Ipv4Addr>>,
    ipv6: Table<Box<str>, RecordSet<Ipv6Addr>>,
    ptr: Table<IpAddr, RecordSet<Box<str>>>,
    sealed: bool,
}

impl std::fmt::Debug for MailDns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MailDns")
            .field("sealed", &self.sealed)
            .finish_non_exhaustive()
    }
}

/// A name with no record, spelled the way a resolver would report NXDOMAIN.
fn absent() -> Error {
    Error::Dns(DnsError::RecordNotFound(ResponseCode::NXDomain))
}

fn sealed_txt() -> Option<Txt> {
    Some(Txt::Error(absent()))
}

/// No answer a record-set table can give is honest; see `resolver_cache!`.
fn decline<T>() -> Option<RecordSet<T>> {
    None
}

impl MailDns {
    /// Answers what it can and declines the rest. Pair it with a resolver
    /// pointed somewhere that cannot answer: a declined lookup still reaches
    /// the resolver, and only the resolver's own reach bounds it.
    pub fn sealed() -> Self {
        Self {
            sealed: true,
            ..Self::default()
        }
    }

    /// A name a conformance run has pinned. `value` is whatever `Txt` holds —
    /// a parsed SPF record, a DKIM key, or an `Error` to pin a refusal.
    pub fn txt_add(&self, name: impl AsRef<str>, value: impl Into<Txt>) {
        self.txt.put(fqdn(name.as_ref()), value.into());
    }

    pub fn ipv4_add(&self, name: impl AsRef<str>, value: Vec<Ipv4Addr>) {
        self.ipv4.put(fqdn(name.as_ref()), records(value));
    }

    pub fn ipv6_add(&self, name: impl AsRef<str>, value: Vec<Ipv6Addr>) {
        self.ipv6.put(fqdn(name.as_ref()), records(value));
    }

    pub fn ptr_add(&self, addr: IpAddr, value: Vec<String>) {
        self.ptr
            .put(addr, records(value.into_iter().map(Into::into).collect()));
    }

    pub fn mx_add(&self, name: impl AsRef<str>, value: Vec<MX>) {
        self.mx.put(fqdn(name.as_ref()), records(value));
    }
}

/// `mail-auth` keys every cache by the fully-qualified name, so a seeded
/// `mx1.foobar.org` has to match the `mx1.foobar.org.` it will ask for.
fn fqdn(name: &str) -> Box<str> {
    let name = name.trim_end_matches('.').to_ascii_lowercase();
    format!("{name}.").into_boxed_str()
}

fn records<T>(rrset: Vec<T>) -> RecordSet<T> {
    RecordSet {
        rrset: rrset.into(),
        dnssec_status: DnssecStatus::Secure,
    }
}

/// Writes are dropped: sealed, the table is the whole world; open, the
/// resolver behind it keeps its own cache.
///
/// A sealed miss answers only where the answer is unambiguous. `Txt` carries
/// an `Error`, so it can say "no such record". A `RecordSet` cannot: an empty
/// one means "the name exists with no records of this type", and `mail-auth`'s
/// `exists:` mechanism tests presence with `get(..).is_some()`, so answering
/// would assert every unseeded name exists and turn a `-all` refusal into a
/// pass. Those tables decline, so a sealed table must be paired with a
/// resolver that cannot leave the host.
// ponytail: no second cache in front of hickory until a measurement asks for
// one; adding it here would need TTL eviction to stay bounded.
macro_rules! resolver_cache {
    ($key:ty, $value:ty, $table:ident, $miss:expr) => {
        impl ResolverCache<$key, $value> for MailDns {
            fn get<Q>(&self, name: &Q) -> Option<$value>
            where
                $key: Borrow<Q>,
                Q: Hash + Eq + ?Sized,
            {
                self.$table
                    .get(name)
                    .or_else(|| if self.sealed { $miss() } else { None })
            }

            fn remove<Q>(&self, name: &Q) -> Option<$value>
            where
                $key: Borrow<Q>,
                Q: Hash + Eq + ?Sized,
            {
                self.$table.remove(name)
            }

            fn insert(&self, _: $key, _: $value, _: Instant) {}
        }
    };
}

resolver_cache!(Box<str>, Txt, txt, sealed_txt);
resolver_cache!(Box<str>, RecordSet<MX>, mx, decline);
resolver_cache!(Box<str>, RecordSet<Ipv4Addr>, ipv4, decline);
resolver_cache!(Box<str>, RecordSet<Ipv6Addr>, ipv6, decline);
resolver_cache!(IpAddr, RecordSet<Box<str>>, ptr, decline);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_open_table_misses_so_the_resolver_behind_it_answers() {
        let dns = MailDns::default();
        assert!(ResolverCache::<Box<str>, Txt>::get(&dns, "nothing.example.").is_none());
        // Production may pin a record without sealing the world.
        dns.txt_add("seeded.example", Txt::Error(absent()));
        assert!(ResolverCache::<Box<str>, Txt>::get(&dns, "seeded.example.").is_some());
    }

    #[test]
    fn a_sealed_txt_miss_is_nxdomain_but_a_record_set_miss_declines() {
        let dns = MailDns::sealed();
        let miss: Option<Txt> = ResolverCache::get(&dns, "nobody.example.");
        assert!(
            matches!(
                miss,
                Some(Txt::Error(Error::Dns(DnsError::RecordNotFound(_))))
            ),
            "a sealed TXT miss must read as NXDOMAIN, not as a cache miss"
        );
        let ptr: Option<RecordSet<Box<str>>> =
            ResolverCache::get(&dns, &IpAddr::from([10, 0, 0, 1]));
        assert!(ptr.is_none(), "a record-set miss must decline, not answer");
        let a: Option<RecordSet<Ipv4Addr>> = ResolverCache::get(&dns, "nobody.example.");
        assert!(a.is_none());
    }

    #[test]
    fn seeding_is_keyed_the_way_mail_auth_asks() {
        // A seed that does not match the fully-qualified name is a silent miss.
        let dns = MailDns::sealed();
        dns.ipv4_add("MX1.FooBar.org", vec![Ipv4Addr::new(10, 0, 0, 1)]);
        let found: Option<RecordSet<Ipv4Addr>> = ResolverCache::get(&dns, "mx1.foobar.org.");
        assert_eq!(
            found.map(|set| set.rrset.to_vec()),
            Some(vec![Ipv4Addr::new(10, 0, 0, 1)])
        );
    }
}
