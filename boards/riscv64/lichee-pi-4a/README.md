# Sipeed Lichee Pi 4A — RV64 bring-up

- SoC: T-Head TH1520 (RV64, C910 cores)
- Console candidate: UART0 at MMIO `0xffe7014000`, compatible `snps,dw-apb-uart`.
- Firmware: inspect the board's actual boot firmware and SBI services before defining the kernel entry contract.
- Device tree: consume the **actual firmware-provided FDT** first. TH1520 has reserved memory regions; don't treat address zero as ordinary allocatable RAM without parsing the FDT.
- Required next steps: DesignWare APB UART driver, TH1520 clock/reset dependencies, interrupt-controller/timer/SBI discovery, then storage and Ethernet.

**Status: scaffold only — not boot-tested.** No prebuilt DTB is claimed by this file.