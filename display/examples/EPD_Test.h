#ifndef _EPD_TEST_H_
#define _EPD_TEST_H_

#include "DEV_Config.h"
#include "GUI_Paint.h"
#include "GUI_BMPfile.h"
#include "ImageData.h"
#include "Debug.h"
#include <stdlib.h> // malloc() free()



int drawToBuffer(int height, int width, const char *text, int *allignment, int size, int position, int startPoint, int yPos);
char *splitCharArray(char *text, int versatz, int maxLength);
int EPD_4in2_V2_test(void);
int EPD_4in2bc_test(void);
int EPD_4in2b_V2_test(void);
#endif
