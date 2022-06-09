toDisplay:toDisplay.o sh1106.o
	gcc -Wall -o oled toDisplay.o sh1106.o -lbcm2835
sh1106.o:sh1106.c sh1106.h
	gcc -Wall -c sh1106.c -lbcm2835
toDisplay.o:toDisplay.c sh1106.h 
	gcc -Wall -c toDisplay.c -lbcm2835
lib:toDisplay.o sh1106.o
	ar rcs libdisplay.a *.o
clean:
	rm sh1106.o toDisplay.o oled libdisplay.a