#include <bcm2835.h>
#include <stdio.h>
#include <time.h>
#include "sh1106.h"

int toDisplay(char *message){
    if(!bcm2835_init())
    {
        return -1;
    }

    SH1106_begin();
    bcm2835_delay(2000);
    SH1106_clear();

    SH1106_string(0, 0, message, 12, 1); 
    
    SH1106_display();

    bcm2835_spi_end();
    bcm2835_close();

    return 0;
}
/*
int main(int argc, char **argv)
{
    char stringDisplay[20];
    printf("Enter string: ");
    scanf("%s", stringDisplay);
    toDisplay(stringDisplay);
}
*/