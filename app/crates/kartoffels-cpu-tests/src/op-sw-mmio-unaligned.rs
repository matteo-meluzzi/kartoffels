#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 0x08000002
        sw x0, 0(x1)
    "#
}

/*
 * err = unaligned mmio store on 0x08000002+4
 */
