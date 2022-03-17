# ThornOS

A UNIX like Operating System, written from the ground up in Rust

# Build Dependencies

Bootimage

# Acknowledgements

The following have been a source of learning and inspiration to ThornOS

- [Philipp Oppermann's Blog](https://os.phil-opp.com/)
- [Operating Systems: Three Easy Pieces](https://pages.cs.wisc.edu/~remzi/OSTEP/)
- [SerenityOS](https://github.com/SerenityOS/serenity)
- [xv6](https://pdos.csail.mit.edu/6.828/2019/xv6.html)

# TODOs

Some general TODOs for now

- Pre commit hook
- CI hookup
- Improve test framework (panic tests)
- Design syscall interface
- ELF loader
- Shell
- Filesystem

Ideally ThornOS will have zero external dependencies. Some core components to reimplement:

- bootloader
- x86 crate
  - Phys Frame
  - Phys Addr
  - Interrupt Descriptor Table
- spinlocks
- uart_16550
- keyboard scancodes
- pic8259