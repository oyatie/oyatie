fn endpoint_has_build_metadata(start: &AdvisoryRangeStartV1, end: &AdvisoryRangeEndV1) -> bool {
    matches!(start, AdvisoryRangeStartV1::Introduced(version) if version.has_build_metadata())
        || matches!(
            end,
            AdvisoryRangeEndV1::Fixed(version)
                | AdvisoryRangeEndV1::LastAffected(version)
                if version.has_build_metadata()
        )
}

fn valid_interval(start: &AdvisoryRangeStartV1, end: &AdvisoryRangeEndV1) -> bool {
    let AdvisoryRangeStartV1::Introduced(start) = start else {
        return true;
    };
    match end {
        AdvisoryRangeEndV1::Unbounded => true,
        AdvisoryRangeEndV1::Fixed(end) => start.precedence_cmp(end) == std::cmp::Ordering::Less,
        AdvisoryRangeEndV1::LastAffected(end) => {
            start.precedence_cmp(end) != std::cmp::Ordering::Greater
        }
    }
}

fn encode_range_start(hash: &mut CanonicalHasherV1, start: &AdvisoryRangeStartV1) {
    match start {
        AdvisoryRangeStartV1::Beginning => hash.tag(0),
        AdvisoryRangeStartV1::Introduced(version) => {
            hash.tag(1);
            hash.digest(version.identity_sha256());
        }
    }
}

fn encode_range_end(hash: &mut CanonicalHasherV1, end: &AdvisoryRangeEndV1) {
    match end {
        AdvisoryRangeEndV1::Unbounded => hash.tag(0),
        AdvisoryRangeEndV1::Fixed(version) => {
            hash.tag(1);
            hash.digest(version.identity_sha256());
        }
        AdvisoryRangeEndV1::LastAffected(version) => {
            hash.tag(2);
            hash.digest(version.identity_sha256());
        }
    }
}

fn compare_ranges(
    left: &CargoAffectedRangeV1,
    right: &CargoAffectedRangeV1,
) -> std::cmp::Ordering {
    compare_starts(&left.start, &right.start)
        .then_with(|| left.identity_sha256.cmp(&right.identity_sha256))
}

fn compare_starts(
    left: &AdvisoryRangeStartV1,
    right: &AdvisoryRangeStartV1,
) -> std::cmp::Ordering {
    match (left, right) {
        (AdvisoryRangeStartV1::Beginning, AdvisoryRangeStartV1::Beginning) => {
            std::cmp::Ordering::Equal
        }
        (AdvisoryRangeStartV1::Beginning, _) => std::cmp::Ordering::Less,
        (_, AdvisoryRangeStartV1::Beginning) => std::cmp::Ordering::Greater,
        (
            AdvisoryRangeStartV1::Introduced(left),
            AdvisoryRangeStartV1::Introduced(right),
        ) => left
            .precedence_cmp(right)
            .then_with(|| left.as_str().as_bytes().cmp(right.as_str().as_bytes())),
    }
}

fn ends_before_start(end: &AdvisoryRangeEndV1, start: &AdvisoryRangeStartV1) -> bool {
    let AdvisoryRangeStartV1::Introduced(start) = start else {
        return false;
    };
    match end {
        AdvisoryRangeEndV1::Unbounded => false,
        AdvisoryRangeEndV1::Fixed(end) => {
            end.precedence_cmp(start) != std::cmp::Ordering::Greater
        }
        AdvisoryRangeEndV1::LastAffected(end) => {
            end.precedence_cmp(start) == std::cmp::Ordering::Less
        }
    }
}
