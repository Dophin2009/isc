NASM = nasm
LD = ld

default: hello

%.o : %.s
	$(NASM) -f elf64 -o $*.o $<

% : %.o
	$(LD) -o $* $<
