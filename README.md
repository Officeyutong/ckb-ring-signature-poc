# Ring Signature on CKB test

Test how many cycles are required to perform a ring signature check on ckb-vm

Refer to Makefile and *.rs for more details.

**NOT FOR PRODUCTION, ONLY PROOF OF CONCEPT**

Files in /src are ring signature for different signature algorithms
- `ecc.rs`: Implementation from https://github.com/beritani/ring-signatures
- `rsa.rs`: Original RSA implementation
- `rsa2.rs`: Implementation from https://github.com/beritani/ring-signatures, using RSA instead of ECC
- `rsa2_linkable.rs`: Implementation from https://github.com/beritani/ring-signatures (blSAG), using RSA instead of ECC. This would be used as the final implementation
