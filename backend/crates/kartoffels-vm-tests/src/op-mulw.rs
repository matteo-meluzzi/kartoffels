#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_vm_tests::test! {
    r#"
    .global _start
    .attribute arch, "rv64im"

    _start:
        li x1, 0xb504f334
        li x2, -0xb504f332
        mulw x3, x1, x2

        li x1, -0xb504f332
        li x2, -0xb504f332
        mulw x4, x1, x2

        li x1, -0xb504f332
        li x2, 0xb504f334
        mulw x5, x1, x2

        li x1, -0x8000000000000000
        li x2, 0xb504f334
        mulw x6, x1, x2
        ebreak
    "#
}

/*
 * x3 = 0x615e23d8
 * x4 = 0x3497f5c4
 * x5 = 0x615e23d8
 * x6 = 0x0
 */
