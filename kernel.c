void testfunc() {
	
}

void kmain() {
	char *video_mem = (char*) 0xb8000;

	for(int i = 0; i < 100; i++) {
		video_mem[2*i] = 'X';	
	}
	
	while(1)
		asm volatile(
			"nop"
		);
}
