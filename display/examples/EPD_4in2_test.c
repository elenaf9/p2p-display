#include "EPD_Test.h"
#include "EPD_4in2.h"
#include <string.h>
#include <time.h>

UBYTE *BlackImage;

int lineSpacing = 10;

void initDisplay(void){

    printf("EPD_4IN2_test Demo\r\n");
    if(DEV_Module_Init()!=0){
        return -1;
    }

    printf("e-Paper Init and Clear...\r\n");
    EPD_4IN2_Init_Fast();

    //Create a new image cache
    /* you have to edit the startup_stm32fxxx.s file and set a big enough heap size */
    UWORD Imagesize = ((EPD_4IN2_WIDTH % 8 == 0)? (EPD_4IN2_WIDTH / 8 ): (EPD_4IN2_WIDTH / 8 + 1)) * EPD_4IN2_HEIGHT;
    if((BlackImage = (UBYTE *)malloc(Imagesize)) == NULL) {
        printf("Failed to apply for black memory...\r\n");
        return -1;
    }
    printf("Paint_NewImage\r\n");
    Paint_NewImage(BlackImage, EPD_4IN2_WIDTH, EPD_4IN2_HEIGHT, 270, WHITE);

    EPD_4IN2_Clear();
    Paint_SelectImage(BlackImage);
    Paint_Clear(WHITE);

}

int drawToBuffer(int height, int width, const char *text, int *allignment, int size, int position, int startPoint, int yPos)
{
    
    int maxChar16 = EPD_4IN2_HEIGHT*size/100/Font16.Width;
    int maxChar24 = EPD_4IN2_HEIGHT*size/100/Font24.Width;

    //set fontsize
    sFONT toDraw;
    if(position < width){
        toDraw = Font24;
    } else {
        toDraw = Font16;
    }
    //cal yPos
    if(position < height){
        yPos = lineSpacing;
    }

    //cal yPos
    int xPos = (EPD_4IN2_HEIGHT/100)*startPoint;


    if(height == 1 && width == 1){
        xPos = (EPD_4IN2_HEIGHT/2)-((strlen(text)*Font24.Width)/2);
        if(xPos < 0){
            xPos = 0;
        }
        if(strlen(text)*Font24.Width>EPD_4IN2_HEIGHT){
            yPos = ((EPD_4IN2_WIDTH-(Font24.Height*((strlen(text)*Font24.Width/300)+1)))/2);
        } else {
            yPos = ((EPD_4IN2_WIDTH-Font24.Height)/2);
        }
        Paint_DrawString_EN(xPos, yPos, text, &Font24, WHITE, BLACK);
    } else {
        printf("text: %s yPos: %d \n",text,yPos);
        if(position < height){
            int textWidth = 1+(strlen(text)*Font24.Width)/(size*EPD_4IN2_HEIGHT/100);
            for(int i = 0; i < textWidth; i++){
                char *this = splitCharArray(text,(i*maxChar24),maxChar24);
                Paint_DrawString_EN(xPos, yPos+i*Font24.Height, this, &Font24, WHITE, BLACK);
            }
            return yPos+(textWidth)*Font24.Height+lineSpacing;
        } else {
            int textWidth = 1+(strlen(text)*Font16.Width)/(size*EPD_4IN2_HEIGHT/100);
            for(int i = 0; i < textWidth; i++){
                char *this = splitCharArray(text,(i*maxChar16),maxChar16);
                Paint_DrawString_EN(xPos, yPos+i*Font16.Height, this, &Font16, WHITE, BLACK);
                
            }
            return yPos+(textWidth)*Font16.Height+lineSpacing;
        }
    }
}

char *splitCharArray(char *text, int versatz, int maxLength){
    char *returnChar = malloc(maxLength);
    for(int i = 0; i < maxLength;i++){
        returnChar[i] = text[i+versatz];
    }
    return strdup(returnChar);
}

void flushToDisplay(void){

    EPD_4IN2_Display(BlackImage);
    printf("Goto Sleep...\r\n");
    EPD_4IN2_Sleep();
    free(BlackImage);
    BlackImage = NULL;
    DEV_Delay_ms(1000);//important, at least 2s
    // close 5V
    printf("close 5V, Module enters 0 power consumption ...\r\n");
    DEV_Module_Exit();
}