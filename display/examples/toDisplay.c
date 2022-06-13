#include <stdlib.h>     //exit()
#include <signal.h>     //signal()
#include "EPD_Test.h"   //Examples

void  Handler(int signo)
{
    //System Exit
    printf("\r\nHandler:exit\r\n");
    DEV_Module_Exit();

    exit(0);
}



int toDisplay(char Text[]){
    signal(SIGINT, Handler);
    EPD_toDisplay();
    char *token = strtok(Text, "|");
    int i = 0;
    while (token != NULL)
    {
        char *toPrint;
        toPrint = token;
        EPD_toDisplay_draw(i,toPrint);
        token = strtok(NULL, "|");
        i = i + 1;
    }
    EPD_toDisplay_flush();
    return 0;
}
