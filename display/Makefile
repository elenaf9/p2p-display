DIR_Config   = ./lib/Config
DIR_EPD      = ./lib/e-Paper
DIR_FONTS    = ./lib/Fonts
DIR_GUI      = ./lib/GUI
DIR_Examples = ./examples
DIR_BIN      = ./bin

OBJ_C = $(wildcard ${DIR_EPD}/*.c ${DIR_GUI}/*.c ${DIR_Examples}/*.c ${DIR_FONTS}/*.c )
OBJ_O = $(patsubst %.c,${DIR_BIN}/%.o,$(notdir ${OBJ_C}))
RPI_DEV_C = $(wildcard $(DIR_BIN)/dev_hardware_SPI.o $(DIR_BIN)/RPI_sysfs_gpio.o $(DIR_BIN)/DEV_Config.o )

DEBUG = -D DEBUG

USELIB_RPI = USE_BCM2835_LIB
# USELIB_RPI = USE_WIRINGPI_LIB
# USELIB_RPI = USE_DEV_LIB

LIB_RPI=-Wl,--gc-sections -I./lib/include/ -L.
ifeq ($(USELIB_RPI), USE_BCM2835_LIB)
    LIB_RPI += -lbcm2835 -lm 
else ifeq ($(USELIB_RPI), USE_WIRINGPI_LIB)
    LIB_RPI += -lwiringPi -lm 
else ifeq ($(USELIB_RPI), USE_DEV_LIB)
    LIB_RPI += -lm 
endif
DEBUG_RPI = -D $(USELIB_RPI) -D RPI

TARGET = epd
CC = gcc
AR = ar
MSG = -g -O -ffunction-sections -fdata-sections -Wall
CFLAGS += $(MSG)

$(shell mkdir -p $(DIR_BIN))

${DIR_BIN}/%.o:$(DIR_Examples)/%.c
	$(CC) $(CFLAGS) -c  $< -o $@ -I $(DIR_Config) -I $(DIR_GUI) -I $(DIR_EPD) $(DEBUG)
    
${DIR_BIN}/%.o:$(DIR_EPD)/%.c
	$(CC) $(CFLAGS) -c  $< -o $@ -I $(DIR_Config) $(DEBUG)
    
${DIR_BIN}/%.o:$(DIR_FONTS)/%.c 
	$(CC) $(CFLAGS) -c  $< -o $@ $(DEBUG)
    
${DIR_BIN}/%.o:$(DIR_GUI)/%.c
	$(CC) $(CFLAGS) -c  $< -o $@ -I $(DIR_Config) $(DEBUG)

.PHONY : RPI clean RPI_DEV RPI_epd lib

RPI: RPI_DEV RPI_epd

RPI_epd: RPI_epd_lib
	$(CC) $(CFLAGS) -D RPI $(OBJ_O) $(RPI_DEV_C) -o $(TARGET) $(LIB_RPI)

RPI_epd_lib: ${OBJ_O}

RPI_DEV:
	$(CC) $(CFLAGS) $(DEBUG_RPI) -c  $(DIR_Config)/dev_hardware_SPI.c -o $(DIR_BIN)/dev_hardware_SPI.o $(LIB_RPI) $(DEBUG)
	$(CC) $(CFLAGS) $(DEBUG_RPI) -c  $(DIR_Config)/RPI_sysfs_gpio.c -o $(DIR_BIN)/RPI_sysfs_gpio.o $(LIB_RPI) $(DEBUG)
	$(CC) $(CFLAGS) $(DEBUG_RPI) -c  $(DIR_Config)/DEV_Config.c -o $(DIR_BIN)/DEV_Config.o $(LIB_RPI) $(DEBUG)

clean:
	rm -rf $(DIR_BIN)
	rm -f $(TARGET)

lib: RPI_DEV RPI_epd_lib
	$(AR) rcs libdisplay.a $(DIR_BIN)/*.o
