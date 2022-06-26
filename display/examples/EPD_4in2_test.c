#include "EPD_Test.h"
#include "EPD_4in2.h"
#include <string.h>
#include <time.h> 
#include <stdbool.h>

UBYTE *BlackImage;

int sizeLines;
int sizeRows;
int widthRows[];

int drawnToX;
int borderOffset = 10;

int EPD_toDisplay(int sizeL, int sizeR,int percWidth[]){

    int widthRows[sizeR];
    for(int i = 0; i < sizeR; i++){
        widthRows[i] = percWidth[i];
    }


    sizeLines = sizeL;
    sizeRows = sizeR;

    for(int a = 0; a < sizeR; a++){
        printf(" percWidth: %d %d | ",a,widthRows[a]);
    }
    printf("\n");
 
    if(DEV_Module_Init()!=0){
        return -1;
    }

    //Create a new image cache
    //UBYTE *BlackImage;
    /* you have to edit the startup_stm32fxxx.s file and set a big enough heap size */
    UWORD Imagesize = ((EPD_4IN2_WIDTH % 8 == 0)? (EPD_4IN2_WIDTH / 8 ): (EPD_4IN2_WIDTH / 8 + 1)) * EPD_4IN2_HEIGHT;
    if((BlackImage = (UBYTE *)malloc(Imagesize)) == NULL) {
        printf("Failed to apply for black memory...\r\n");
        return -1;
    }

    Paint_NewImage(BlackImage, EPD_4IN2_WIDTH, EPD_4IN2_HEIGHT, 270, WHITE);
    EPD_4IN2_Init_Fast();

    EPD_4IN2_Clear();
    Paint_SelectImage(BlackImage);
    Paint_Clear(WHITE);

    drawnToX = 0;

    return 0;
}

int calcTextPos(int index, int width, char *input, sFONT frontType){
    int pos = ((300-(2*borderOffset)) / (width+1)) * (index%width);

    
    if(index%width == 0){
        drawnToX = 0;
    }
    
    if(pos+borderOffset>drawnToX){
        drawnToX = frontType.Width*strlen(input)+pos;
        return pos+borderOffset;
    } else {
        pos = drawnToX + frontType.Width;
        drawnToX = frontType.Width*strlen(input)+drawnToX+frontType.Width;
        return pos;
    }
}

int EPD_toDisplay_draw(int position, char *input){
    bool correctPercent = true;
    bool firstLine = true;

    if(sizeLines == 1){
        printf("this!\n");


        Paint_DrawString_EN((300-(Font24.Width*strlen(strdup(input))))/2, (400-Font24.Height)/2, input, &Font24, WHITE, BLACK);
        return 0;
    }

    int xPos = 0;
    if(position < sizeRows-1){
        xPos = calcTextPos(position,sizeRows,input,Font24);
    } else {
        xPos = calcTextPos(position,sizeRows,input,Font16);
    }

    for(int a = 0; a < sizeRows; a++){
        if(widthRows[a] > 100){
            correctPercent = false;
        }
    }

    printf("init: %s | x = %d | y = %d | len: %d | xPos: %d | posi: %d\n",input,sizeLines,sizeRows,strlen(input),xPos,position);
    printf("\n");

    if(!correctPercent){
        printf("yes\n");
        if(position == 0){
            Paint_DrawString_EN(xPos, 0, input, &Font24, WHITE, BLACK);
        }
        if(position == 1){
            Paint_DrawString_EN(xPos, 0, input, &Font24, WHITE, BLACK);
        }
        if(position == 2){
            Paint_DrawString_EN(xPos, 0, input, &Font24, WHITE, BLACK);
        }
        if(position > 2 && position < 6){
            Paint_DrawString_EN(xPos, 120, input, &Font16, WHITE, BLACK);
        }
        if(position > 5 && position < 9){
            Paint_DrawString_EN(xPos, 180, input, &Font16, WHITE, BLACK);
        }
        if(position > 8 && position < 12){
            Paint_DrawString_EN(xPos, 240, input, &Font16, WHITE, BLACK);
        }
    }
    
    

    return 0;
}

int EPD_toDisplay_flush(){

    EPD_4IN2_Display(BlackImage);

    DEV_Delay_ms(2000);//important, at least 2s
    
    DEV_Module_Exit();

    free(BlackImage);
    BlackImage = NULL;

    return 0;

}