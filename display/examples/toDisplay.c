#include <stdio.h>
#include <string.h>
#include <stdlib.h>     //exit()
#include <signal.h>     //signal()
#include "EPD_Test.h"   //Examples

int qLines;
int qRows;
int percentRow[];

void  Handler(int signo)
{
    //System Exit
    printf("\r\nHandler:exit\r\n");
    DEV_Module_Exit();

    exit(0);
}

/*
int main(){
    char str[1024] = "r|5|l|10|c|85\nT9|K.12|Mon\n08:00 -|10:00|SWP\n10:00 -|14:00|Unbenutzt\n14:00 -|16:00|Techno";
    //char str[50] = "r|5|l|10|c|85\nT9||\n";
    //printf("Enter a string : ");
    //gets(str);
    toDisplay(str);
}
*/

int toDisplay(char Text[]){
    signal(SIGINT, Handler);
    

    char multiCharList[10][50];

    int lineNr = 0;
    char *line = strtok(Text, "\n");
    while (line != NULL)
    {
        //printf("this %d %s \n",lineNr,line);
        strcpy(multiCharList[lineNr],strdup(line));

        line = strtok(NULL, "\n");
        lineNr = lineNr + 1;
    }
    qLines = lineNr;
    getPositionInfo(multiCharList[0]);

    EPD_toDisplay(qLines-1,qRows,percentRow);

    int b = 0;
    for(int a = 0; a < lineNr-1; a++){
        b = DisplayFunction(multiCharList[a+1],b*a);
    }
    
    EPD_toDisplay_flush();
    return 0;
}

void getPositionInfo(char line[]){

    char mulitList[10][50];

    char *token = strtok(line, "|");
    int i = 0;
    while (token != NULL)
    {
        strcpy(mulitList[i],strdup(token));
        token = strtok(NULL, "|");
        i = i + 1;
    }

    printf("width: %d \n",i/2);
    int widthL[i/2];
    int allignment[i/2];

    percentRow[15];
    qRows = i/2;

    for(int a = 0; a < i; a++){
        if(strpbrk("l", mulitList[a]) != 0){
            allignment[a/2] = 0;
        } else {
            if(strpbrk("c", mulitList[a]) != 0){
                allignment[a/2] = 1;
            } else {
                if(strpbrk("r", mulitList[a]) != 0){
                    allignment[a/2] = 2;
                } else {
                    percentRow[a/2] = atoi(mulitList[a]);
                }
            }
        }
    }
}

int DisplayFunction(char line[], int offset){

    char *token = strtok(line, "|");
    int i = 0;
    while (token != NULL)
    {
        EPD_toDisplay_draw(i+offset,token);
        token = strtok(NULL, "|");
        i = i + 1;
    }

    return i;
}