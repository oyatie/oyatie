#!/usr/bin/env bash
# diagnose.sh — collect everything needed to debug the openbao.service start failure.
# Run with sudo so we can read /etc/openbao/.
set +e

echo "=== A. perms on /etc/openbao ==="
ls -la /etc/openbao/ 2>&1

echo
echo "=== B. apparmor status (Debian default) ==="
aa-status 2>&1 | head -20

echo
echo "=== C. selinux status (just in case) ==="
sestatus 2>&1 | head -5

echo
echo "=== D. openbao user perms test ==="
sudo -u openbao test -r /etc/openbao/openbao.hcl && echo "openbao CAN read openbao.hcl" || echo "openbao CANNOT read openbao.hcl"
sudo -u openbao stat /etc/openbao/openbao.hcl 2>&1 | head -5

echo
echo "=== E. run diagnose as openbao ==="
sudo -u openbao /usr/bin/bao operator diagnose -config=/etc/openbao/openbao.hcl 2>&1 | tail -40

echo
echo "=== F. systemctl status ==="
systemctl --no-pager status openbao.service 2>&1 | head -20

echo
echo "=== G. journalctl (last 50) ==="
journalctl -u openbao.service -n 50 --no-pager 2>&1 | tail -50

echo
echo "=== H. data dir state ==="
ls -la /srv/oyatie/openbao /srv/oyatie/openbao/data 2>&1 | head -10
zfs list oyatie-bulk/srv/openbao 2>&1 | head -3
