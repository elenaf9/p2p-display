#include <bcm2835.h>
#include <stdio.h>
#include "sh1106.h"
#define VCCSTATE SH1106_SWITCHCAPVCC
#define WIDTH 128
#define HEIGHT 64
#define PAGES 8

#define RST 19 
#define DC  16
char buffer[1024];

void command(char cmd) {
    bcm2835_gpio_write(DC, LOW);
    bcm2835_spi_transfer(cmd);
}

void SH1106_begin()
{
    bcm2835_gpio_fsel(RST, BCM2835_GPIO_FSEL_OUTP);
    bcm2835_gpio_fsel(DC, BCM2835_GPIO_FSEL_OUTP);

    bcm2835_spi_begin();
    bcm2835_spi_setBitOrder(BCM2835_SPI_BIT_ORDER_MSBFIRST);     //The default
    bcm2835_spi_setDataMode(BCM2835_SPI_MODE0);                  //The default
    bcm2835_spi_setClockDivider(BCM2835_SPI_CLOCK_DIVIDER_2048);  //The default
    bcm2835_spi_chipSelect(BCM2835_SPI_CS0);                     //The default
    bcm2835_spi_setChipSelectPolarity(BCM2835_SPI_CS0, LOW);     //the default

    bcm2835_gpio_write(RST, HIGH);
    bcm2835_delay(10);
    bcm2835_gpio_write(RST, LOW);
    bcm2835_delay(10);
    bcm2835_gpio_write(RST, HIGH);

    command(0xAE);//--turn off oled panel
    command(0x02);//---set low column address
    command(0x10);//---set high column address
    command(0x40);//--set start line address  Set Mapping RAM Display Start Line (0x00~0x3F)
    command(0x81);//--set contrast control register
    command(0xA0);//--Set SEG/Column Mapping     
    command(0xC0);//Set COM/Row Scan Direction   
    command(0xA6);//--set normal display
    command(0xA8);//--set multiplex ratio(1 to 64)
    command(0x3F);//--1/64 duty
    command(0xD3);//-set display offset    Shift Mapping RAM Counter (0x00~0x3F)
    command(0x00);//-not offset
    command(0xd5);//--set display clock divide ratio/oscillator frequency
    command(0x80);//--set divide ratio, Set Clock as 100 Frames/Sec
    command(0xD9);//--set pre-charge period
    command(0xF1);//Set Pre-Charge as 15 Clocks & Discharge as 1 Clock
    command(0xDA);//--set com pins hardware configuration
    command(0x12);
    command(0xDB);//--set vcomh
    command(0x40);//Set VCOM Deselect Level
    command(0x20);//-Set Page Addressing Mode (0x00/0x01/0x02)
    command(0x02);//
    command(0xA4);// Disable Entire Display On (0xa4/0xa5)
    command(0xA6);// Disable Inverse Display On (0xa6/a7) 
    command(0xAF);//--turn on oled panel
}
void SH1106_clear()
{
    int i;
    for(i = 0; i < sizeof(buffer); i++)
    {
        buffer[i] = 0;
    }
}
void SH1106_pixel(int x, int y, char color)
{
    if(x > WIDTH || y > HEIGHT)return ;
    if(color)
        buffer[x+(y/8)*WIDTH] |= 1<<(y%8);
    else
        buffer[x+(y/8)*WIDTH] &= ~(1<<(y%8));
}
void SH1106_char1616(uint8_t x, uint8_t y, uint8_t chChar)
{
    uint8_t i, j;
    uint8_t chTemp = 0, y0 = y, chMode = 0;

    for (i = 0; i < 32; i ++) {
        chTemp = Font1612[chChar - 0x30][i];
        for (j = 0; j < 8; j ++) {
            chMode = chTemp & 0x80? 1 : 0; 
            SH1106_pixel(x, y, chMode);
            chTemp <<= 1;
            y ++;
            if ((y - y0) == 16) {
                y = y0;
                x ++;
                break;
            }
        }
    }
}

void SH1106_char3216(uint8_t x, uint8_t y, uint8_t chChar)
{
    uint8_t i, j;
    uint8_t chTemp = 0, y0 = y, chMode = 0;

    for (i = 0; i < 64; i ++) {
        chTemp = Font3216[chChar - 0x30][i];
        for (j = 0; j < 8; j ++) {
            chMode = chTemp & 0x80? 1 : 0; 
            SH1106_pixel(x, y, chMode);
            chTemp <<= 1;
            y ++;
            if ((y - y0) == 32) {
                y = y0;
                x ++;
                break;
            }
        }
    }
}
void SH1106_char(unsigned char x, unsigned char y, char acsii, char size, char mode)
{
    unsigned char i, j, y0=y;
    char temp;
    unsigned char ch = acsii - ' ';
    for(i = 0;i<size;i++) {
        if(size == 12)
        {
            if(mode)temp=Font1206[ch][i];
            else temp = ~Font1206[ch][i];
        }
        else 
        {            
            if(mode)temp=Font1608[ch][i];
            else temp = ~Font1608[ch][i];
        }
        for(j =0;j<8;j++)
        {
            if(temp & 0x80) SH1106_pixel(x, y, 1);
            else SH1106_pixel(x, y, 0);
            temp <<=1;
            y++;
            if((y-y0)==size)
            {
                y = y0;
                x ++;
                break;
            }
        }
    }
}
void SH1106_string(uint8_t x, uint8_t y, const char *pString, uint8_t Size, uint8_t Mode)
{
    while (*pString != '\0') {       
        if (x > (WIDTH - Size / 2)) {
            x = 0;
            y += Size;
            if (y > (HEIGHT - Size)) {
                y = x = 0;
            }
        }
        
        SH1106_char(x, y, *pString, Size, Mode);
        x += Size / 2;
        pString ++;
    }
}

void SH1106_bitmap(unsigned char x, unsigned char y, const unsigned char *pBmp, 
                        unsigned char chWidth, unsigned char chHeight)
{
    unsigned char i, j, byteWidth = (chWidth + 7)/8;
    for(j = 0; j < chHeight; j++) {
        for(i = 0;i <chWidth;i ++) {
            if(*(pBmp + j*byteWidth + i/8) & (128 >> (i & 7))) {
                SH1106_pixel(x+i, y+j, 1);
            }
        }
    }        
}

void SH1106_display()
{
    uint8_t page;
    char *pBuf = (char *)buffer;
    
    for (page = 0; page < 8; page++) {  
        /* set page address */
        command(0xB0 + page);
        /* set low column address */
        command(0x02); 
        /* set high column address */
        command(0x10); 
        /* write data */
        bcm2835_gpio_write(DC, HIGH);
        bcm2835_spi_transfern(pBuf, WIDTH); 
        pBuf += WIDTH;
    }
}
