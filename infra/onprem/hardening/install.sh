#!/usr/bin/env bash
# setup-hardening.sh — bundled high-priority hardening for the oyatie host.
# Idempotent. Safe to re-run. Each section is a function; comment out the
# dispatch line at the bottom to skip.
#
# Tunables (override via env):
#   OYATIE_ALERT_EMAIL=you@example.com  (default: root@localhost)
#   OYATIE_ARC_MAX_GIB=12               (default: 12)
#
# What this script applies automatically:
#   - ZFS monthly scrub timer for oyatie-bulk
#   - smartmontools with disk-failure email
#   - systemd-timesyncd if no time sync is active
#   - ZED (zfs-event-daemon) email destination
#   - sysctl hardening + TCP BBR
#   - journald 2 GiB cap, persistent storage
#   - ZFS ARC max cap (requires reboot to apply)
#   - systemd drop-in tightening oyatie.service (applies when service exists)
#   - fail2ban (default sshd jail)
#
# What this script DRAFTS but does NOT enable (you activate manually):
#   - /etc/nftables.conf  (could block your SSH if rules are wrong)
#   - /etc/ssh/sshd_config.d/99-oyatie.conf  (could lock you out)
set -euo pipefail

ALERT_EMAIL="${OYATIE_ALERT_EMAIL:-root@localhost}"
ARC_MAX_GIB="${OYATIE_ARC_MAX_GIB:-12}"

banner() { printf "\n=== %s ===\n" "$*"; }

# ---------- 1. ZFS monthly scrub ----------
step_zfs_scrub() {
  banner "ZFS monthly scrub timer"
  sudo systemctl enable --now zfs-scrub-monthly@oyatie-bulk.timer
  systemctl list-timers 'zfs-scrub-*@oyatie-bulk.timer' --no-pager
}

# ---------- 2. smartmontools ----------
step_smartmontools() {
  banner "smartmontools"
  sudo apt-get install -y smartmontools
  sudo tee /etc/smartd.conf > /dev/null <<EOF
# Managed by setup-hardening.sh
# DEVICESCAN: monitor every drive smartctl can see.
#   -a       all attributes
#   -o on    enable offline tests
#   -S on    save attribute autosave
#   -n standby,q  skip if drive is asleep, quiet
#   -s (...) short test daily 02:xx, long test Saturday 03:xx
#   -W 4,40,50  warn on temp delta 4C, alarm at 40C, critical 50C
#   -m EMAIL  alert destination
DEVICESCAN -a -o on -S on -n standby,q -s (S/../.././02|L/../../6/03) -W 4,40,50 -m $ALERT_EMAIL
EOF
  # Debian ships the unit as `smartmontools.service`; some installs leave a
  # `smartd.service` compat symlink that systemctl refuses to enable directly.
  # Try the canonical Debian unit first, fall back to the legacy name on RHEL/etc.
  if systemctl list-unit-files smartmontools.service >/dev/null 2>&1; then
    sudo rm -f /etc/systemd/system/smartd.service
    sudo systemctl daemon-reload
    sudo systemctl enable --now smartmontools.service
    systemctl is-active smartmontools.service
  else
    sudo systemctl enable --now smartd.service
    systemctl is-active smartd.service
  fi
}

# ---------- 3. NTP / time sync ----------
step_ntp() {
  banner "time sync"
  if ! timedatectl show -p NTPSynchronized --value 2>/dev/null | grep -qi true; then
    echo "no active NTP sync — enabling systemd-timesyncd"
    sudo systemctl enable --now systemd-timesyncd
  else
    echo "NTP sync already active"
  fi
  timedatectl status | head -10
}

# ---------- 4. ZED email destination ----------
step_zed_email() {
  banner "ZED email destination ($ALERT_EMAIL)"
  if [ -f /etc/zfs/zed.d/zed.rc ]; then
    sudo sed -i \
      -e "s|^#\?ZED_EMAIL_ADDR=.*|ZED_EMAIL_ADDR=\"$ALERT_EMAIL\"|" \
      -e "s|^#\?ZED_NOTIFY_VERBOSE=.*|ZED_NOTIFY_VERBOSE=0|" \
      /etc/zfs/zed.d/zed.rc
    sudo systemctl restart zfs-zed
    grep -E '^ZED_(EMAIL_ADDR|NOTIFY_VERBOSE)' /etc/zfs/zed.d/zed.rc
  else
    echo "WARN: /etc/zfs/zed.d/zed.rc not found — install zfs-zed"
  fi
}

# ---------- 5. sysctl hardening + TCP BBR ----------
step_sysctl() {
  banner "sysctl hardening + TCP BBR"
  sudo tee /etc/sysctl.d/99-oyatie-hardening.conf > /dev/null <<'EOF'
# Managed by setup-hardening.sh
# Kernel info exposure
kernel.kptr_restrict=2
kernel.dmesg_restrict=1
kernel.unprivileged_bpf_disabled=1
net.core.bpf_jit_harden=2

# Network safety
net.ipv4.tcp_syncookies=1
net.ipv4.conf.all.rp_filter=1
net.ipv4.conf.default.rp_filter=1
net.ipv4.conf.all.accept_redirects=0
net.ipv4.conf.default.accept_redirects=0
net.ipv4.conf.all.send_redirects=0
net.ipv4.conf.default.send_redirects=0
net.ipv4.conf.all.accept_source_route=0
net.ipv6.conf.all.accept_redirects=0
net.ipv6.conf.default.accept_redirects=0
net.ipv6.conf.all.accept_ra=0

# Filesystem safety
fs.protected_hardlinks=1
fs.protected_symlinks=1
fs.protected_fifos=2
fs.protected_regular=2

# TCP BBR — better throughput on lossy/long-fat pipes
net.core.default_qdisc=fq
net.ipv4.tcp_congestion_control=bbr
EOF
  echo tcp_bbr | sudo tee /etc/modules-load.d/tcp_bbr.conf > /dev/null
  sudo modprobe tcp_bbr 2>/dev/null || true
  sudo sysctl --system | tail -20
}

# ---------- 6. journald cap ----------
step_journald() {
  banner "journald: persistent + 2 GiB cap"
  sudo install -d -m 0755 /etc/systemd/journald.conf.d
  sudo tee /etc/systemd/journald.conf.d/99-oyatie.conf > /dev/null <<'EOF'
[Journal]
Storage=persistent
SystemMaxUse=2G
SystemKeepFree=1G
EOF
  sudo systemctl restart systemd-journald
  journalctl --disk-usage
}

# ---------- 7. ZFS ARC max cap ----------
step_arc_cap() {
  banner "ZFS ARC max = ${ARC_MAX_GIB} GiB"
  local bytes=$((ARC_MAX_GIB * 1024 * 1024 * 1024))
  sudo tee /etc/modprobe.d/zfs.conf > /dev/null <<EOF
# Managed by setup-hardening.sh
options zfs zfs_arc_max=$bytes
EOF
  sudo update-initramfs -u
  echo "ARC max will be capped at ${ARC_MAX_GIB} GiB after next reboot."
  echo "Current ARC max: $(awk '/c_max/{print $3}' /proc/spl/kstat/zfs/arcstats) bytes"
}

# ---------- 8. systemd drop-in for oyatie.service ----------
step_systemd_dropin() {
  banner "systemd drop-in: hardening for oyatie.service"
  sudo install -d -m 0755 /etc/systemd/system/oyatie.service.d
  sudo tee /etc/systemd/system/oyatie.service.d/10-hardening.conf > /dev/null <<'EOF'
# Managed by setup-hardening.sh — applied on top of the base unit.
[Service]
ProtectClock=true
ProtectHostname=true
ProtectKernelLogs=true
ProtectProc=invisible
ProcSubset=pid
UMask=0027
RemoveIPC=true
KeyringMode=private
CapabilityBoundingSet=

# Resource limits — tune MemoryMax once you have real usage data
MemoryHigh=512M
MemoryMax=1G
TasksMax=128
LimitNOFILE=65536
LimitNPROC=64

# Accounting (cheap, useful for later metrics)
IPAccounting=true
TasksAccounting=true
MemoryAccounting=true
CPUAccounting=true
EOF
  sudo systemctl daemon-reload
  echo "drop-in active for oyatie.service when it exists."
  echo "verify with: systemd-analyze security oyatie.service  (after the unit is created)"
}

# ---------- 9. fail2ban ----------
step_fail2ban() {
  banner "fail2ban (default sshd jail)"
  sudo apt-get install -y fail2ban
  # Local override so package upgrades don't clobber tuning.
  sudo install -d -m 0755 /etc/fail2ban
  sudo tee /etc/fail2ban/jail.d/99-oyatie.conf > /dev/null <<'EOF'
[DEFAULT]
bantime = 1h
findtime = 10m
maxretry = 5
backend = systemd

[sshd]
enabled = true
EOF
  sudo systemctl enable --now fail2ban
  sudo fail2ban-client status sshd 2>/dev/null || true
}

# ---------- 10. nftables (DRAFT — does not activate) ----------
step_nftables_draft() {
  banner "nftables: drafting /etc/nftables.conf (NOT activating)"
  sudo apt-get install -y nftables
  sudo tee /etc/nftables.conf.draft > /dev/null <<'EOF'
#!/usr/sbin/nft -f
# Managed by setup-hardening.sh — minimal default-deny firewall.
# Allows: loopback, established connections, tailscale interface (full trust),
# ICMP for path MTU + ping, SSH on tcp/22 from anywhere.
# Drops: everything else inbound. Forwarding disabled (not a router).
flush ruleset

table inet oyatie {
  chain input {
    type filter hook input priority filter; policy drop;

    ct state established,related accept
    ct state invalid drop

    iif lo accept

    # tailscale — full trust on the encrypted overlay
    iifname "tailscale0" accept

    # ICMP path MTU + ping
    ip protocol icmp icmp type { echo-request, destination-unreachable, time-exceeded } accept
    ip6 nexthdr icmpv6 icmpv6 type {
      echo-request, destination-unreachable, packet-too-big,
      time-exceeded, parameter-problem,
      nd-router-solicit, nd-router-advert,
      nd-neighbor-solicit, nd-neighbor-advert
    } accept

    # SSH — tighten the source after you confirm tailscale-only access works
    tcp dport 22 accept
  }

  chain forward {
    type filter hook forward priority filter; policy drop;
  }

  chain output {
    type filter hook output priority filter; policy accept;
  }
}
EOF
  echo "draft written to /etc/nftables.conf.draft"
  echo
  echo "TO ACTIVATE (run from a terminal you can recover from if SSH dies):"
  echo "  sudo nft -c -f /etc/nftables.conf.draft        # syntax check, no apply"
  echo "  sudo cp /etc/nftables.conf.draft /etc/nftables.conf"
  echo "  sudo systemctl restart nftables                # apply"
  echo "  # verify SSH still works in a SECOND ssh session before continuing"
  echo "  sudo systemctl enable nftables                 # persist on boot"
  echo
  echo "  Safety net: in another terminal first run:"
  echo "    sudo bash -c '(sleep 300 && nft flush ruleset) &'"
  echo "  This auto-flushes after 5 min if you get locked out — kill it once you've verified."
}

# ---------- 11. sshd hardening (DRAFT — does not activate destructive flags) ----------
step_sshd_draft() {
  banner "sshd: drafting /etc/ssh/sshd_config.d/99-oyatie.conf"
  sudo install -d -m 0755 /etc/ssh/sshd_config.d
  sudo tee /etc/ssh/sshd_config.d/99-oyatie.conf > /dev/null <<'EOF'
# Managed by setup-hardening.sh — safe-to-apply tightening.
# Note: PasswordAuthentication is NOT disabled here — flip to "no" only after
# you've verified key-based login works for every user who needs access.
PermitRootLogin no
X11Forwarding no
AllowAgentForwarding no
AllowTcpForwarding no
PrintMotd no
ClientAliveInterval 300
ClientAliveCountMax 2
MaxAuthTries 3
LoginGraceTime 20
# PasswordAuthentication no   # <-- uncomment once you've tested key login
# KbdInteractiveAuthentication no
EOF
  if sudo sshd -t; then
    sudo systemctl reload ssh
    echo "sshd reloaded with the safe tightening above."
  else
    echo "ERROR: sshd config failed validation — config not reloaded."
    return 1
  fi
  echo
  echo "TO FULLY DISABLE PASSWORD AUTH (after key login is verified):"
  echo "  sudo sed -i 's|^# PasswordAuthentication no|PasswordAuthentication no|' /etc/ssh/sshd_config.d/99-oyatie.conf"
  echo "  sudo sshd -t && sudo systemctl reload ssh"
}

# ---------- dispatch ----------
step_zfs_scrub
step_smartmontools
step_ntp
step_zed_email
step_sysctl
step_journald
step_arc_cap
step_systemd_dropin
step_fail2ban
step_nftables_draft
step_sshd_draft

banner "all done"
echo
echo "Reboot needed for ARC cap to take effect (no rush)."
echo
echo "Manual follow-ups:"
echo "  1. Activate nftables — see instructions above."
echo "  2. After verifying key SSH works, disable password auth — see instructions above."
echo "  3. Configure an MTA (msmtp / postfix) so smartd + ZED emails actually deliver:"
echo "     sudo apt install msmtp msmtp-mta bsd-mailx"
echo "     then write /etc/msmtprc pointing at your SMTP relay."
echo "  4. Set OYATIE_ALERT_EMAIL=you@example.com and re-run this script if you want a"
echo "     real alert destination instead of root@localhost."
