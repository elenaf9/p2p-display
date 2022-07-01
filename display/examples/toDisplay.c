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


int main(void){
    char myString[] = "c|10|r|20|l|70\nl1|l2|l3\na|b|c\nd|e|f\n";

    toDisplay(&myString);

    char myString1[] = "c|10|r|20|l|60|l|10\nl1|l2|l3|l4\na|b|c|d\ne|f|g|h\ni|j|k|l\n";

    toDisplay(&myString1);

    char myString2[] = "r|30|l|70\nl1|l2\na|b\nc|d\n";

    toDisplay(&myString2);

    char myString3[] = "c|100\nl1\na\nb\n";

    toDisplay(&myString3);

    printf("A %s\n",myString);

    return 0;
}


void toDisplay(char *text)
{   
    initDisplay();

    const int height = getAmount(text,"\n");

    signal(SIGINT, Handler);

    char multiList[height][1024];

    int lineNumber = 0;
    char *line = strtok(text, "\n");
    while(line != NULL){
        strcpy(multiList[lineNumber],strdup(line));
        line = strtok(NULL, "\n");
        lineNumber = lineNumber + 1;
    }

    const int width = 1+(getAmount(multiList[0],"|")/2);
    int allignmentList[width][2];

    for(int lineY = 0; lineY < height; lineY++){
        int linePos = 0;
        int startPoint = 0;
        char *rowChar = strtok(multiList[lineY], "|");
        while(rowChar != NULL){
            if(lineY == 0){
                if(strpbrk("l", rowChar) != 0){
                    allignmentList[linePos/2][linePos%2] = 0;
                } else {
                    if(strpbrk("c", rowChar) != 0){
                        allignmentList[linePos/2][linePos%2] = 1;
                    } else {
                        if(strpbrk("r", rowChar) != 0){
                            allignmentList[linePos/2][linePos%2] = 2;
                        } else {
                            allignmentList[linePos/2][linePos%2] = atoi(rowChar);
                        }
                    }
                }
            } else {
                //Print to Display
                printMethod(width,height-1,rowChar,allignmentList[linePos][0],allignmentList[linePos][1],linePos+(lineY-1)*width);
                drawToBuffer(width,height-1,rowChar,allignmentList[linePos][0],allignmentList[linePos][1],linePos+(lineY-1)*width,startPoint);
                startPoint = startPoint + allignmentList[linePos][1];
            }
        
            rowChar = strtok(NULL, "|");
            linePos = linePos + 1;
        }
        printf("\n");
    }

    //drawToBuffer();
    printf("\n");

    flushToDisplay();
}

int getAmount(char *text, char del[]){

    int count = 0;
    const char *tmp = text;
    while(tmp = strstr(tmp, del))
    {
        count++;
        tmp++;
    }

    return count;
}

void printMethod(int height, int width, char *text, int *allignment, int *size, int position){
    printf("Table size %d x %d -- current word: %s --", height, width, text);
    printf(" allignment: %d percentsize: %d position: %d",allignment,size,position);
    printf(" postion in table y %d x %d",(position/height),(position%height));
    printf("\n");
}