# System services

The sys directory contains user-space service contracts, not kernel drivers. Services should communicate through the microkernel IPC boundary after the real Gravity Kernel ABI is integrated. This initial implementation is a host-testable registry; it does not launch processes or perform IPC itself.
