tenancy_ocid = "ocid1.tenancy.oc1..aaaaaaaafapguslxi54jdloww2rhtlyb7fyhf3tqgjm7xpfiwveuy43ltt3a"
region       = "ap-chuncheon-1"
ssh_authorized_keys = [
  "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJ6dcjCZ33c4wU1XaGXLhvDjdabGAQ1YZelM5L37AUwP oyatie@onprem-2026-05-16"
]

// Oracle Linux 10.1 aarch64 (2026-04-30-3) — latest LTS minor, paired with
// the A1.Flex ARM shape. Per user direction 2026-05-16 ('try a1 OL 10 latest';
// 'same with e2 should be ol 10').
// Alternate OL 10 OCIDs (swap-in via tfvars override on capacity failure):
//   OL 10.1 x86_64 (E2.1.Micro fallback): ocid1.image.oc1.ap-chuncheon-1.aaaaaaaa7dt7pyhhltpw2lfpgqvfhwy3b3g6jbbzm3vh5ag3masvvd2bo6ia
stage0_image_ocid          = "ocid1.image.oc1.ap-chuncheon-1.aaaaaaaaxqvh4tk52g3du4527cskiow35ided5vzrr3o4vykmpl54tp43ccq"
stage0_availability_domain = "Iyyn:AP-CHUNCHEON-1-AD-1"

// Shape: VM.Standard.A1.Flex sized to the full Always-Free A1 envelope
// (4 OCPU / 24 GB) per user direction 2026-05-16 ('A1 4 vCPU 24GB ram').
// This consumes the entire A1.Flex Always-Free allocation on this tenancy;
// the 2 E2.1.Micro auxiliary instances (compute-aux.tf) round out the
// always-free fleet.
stage0_shape      = "VM.Standard.A1.Flex"
stage0_ocpus      = 4
stage0_memory_gbs = 24
