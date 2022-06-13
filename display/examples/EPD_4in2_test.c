/*****************************************************************************
* | File      	:   EPD_4in2_test.c
* | Author      :   Waveshare team
* | Function    :   4.2inch e-paper test demo
* | Info        :
*----------------
* |	This version:   V1.0
* | Date        :   2019-06-13
* | Info        :
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documnetation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to  whom the Software is
# furished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in
# all copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS OR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
# THE SOFTWARE.
#
******************************************************************************/
#include "EPD_Test.h"
#include "EPD_4in2.h"
#include <string.h>
#include <time.h> 

UBYTE *BlackImage;

int EPD_toDisplay(){

    printf("EPD_4IN2_test Demo\r\n");
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
    printf("Paint_NewImage\r\n");
    Paint_NewImage(BlackImage, EPD_4IN2_WIDTH, EPD_4IN2_HEIGHT, 270, WHITE);

    printf("e-Paper Init and Clear...\r\n");
    EPD_4IN2_Init_Fast();


    EPD_4IN2_Clear();
    Paint_SelectImage(BlackImage);
    Paint_Clear(WHITE);

    return 0;
}

int EPD_toDisplay_draw(int position, char *input){

    //Paint_SetRotate(270);
    Paint_DrawString_EN(10, 0, "T9/1.12", &Font24, WHITE, BLACK);
    Paint_DrawString_EN(190, 0, "Montag", &Font24, WHITE, BLACK);

    if(position == 0){
        Paint_DrawString_EN(10, 40, "08:00-10:00", &Font16, WHITE, BLACK);
        Paint_DrawString_EN(150, 40, input, &Font16, WHITE, BLACK);
    }
    if(position == 1){
        Paint_DrawString_EN(10, 100, "10:00-12:00", &Font16, WHITE, BLACK);
        Paint_DrawString_EN(150, 100, input, &Font16, WHITE, BLACK);
    }
    if(position == 2){
        Paint_DrawString_EN(10, 160, "12:00-14:00", &Font16, WHITE, BLACK);
        Paint_DrawString_EN(150, 160, input, &Font16, WHITE, BLACK);
    }
    if(position == 3){
        Paint_DrawString_EN(10, 220, "14:00-16:00", &Font16, WHITE, BLACK);
        Paint_DrawString_EN(150, 220, input, &Font16, WHITE, BLACK);
    }
    if(position == 4){
        Paint_DrawString_EN(10, 280, "16:00-18:00", &Font16, WHITE, BLACK);
        Paint_DrawString_EN(150, 280, input, &Font16, WHITE, BLACK);
    }
    if(position == 5){
        Paint_DrawString_EN(10, 340, "18:00-20:00", &Font16, WHITE, BLACK);
        Paint_DrawString_EN(150, 340, "Party", &Font16, WHITE, BLACK);
    }
    if(position == 6){
        printf("test 7\n");
    }
    if(position == 7){
        printf("test 8\n");
    }

    return 0;
}

int EPD_toDisplay_flush(){

    EPD_4IN2_Display(BlackImage);

    DEV_Delay_ms(2000);//important, at least 2s
    
    // close 5V
    printf("close 5V, Module enters 0 power consumption ...\r\n");
    DEV_Module_Exit();

    free(BlackImage);
    BlackImage = NULL;

    return 0;

}