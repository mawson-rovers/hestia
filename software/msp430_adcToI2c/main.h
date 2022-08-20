#ifndef MAIN_H
#define MAIN_H

// GPIO PINS
#define LED_GREEN 0x04
#define LED_YELLOW 0x08
#define HEATER_PIN 0x07

// heater modes
#define HEATER_MODE_OFF 0x00
#define HEATER_MODE_PID 0x01
#define HEATER_MODE_PWM 0x02


/* Initialized the software state machine according to the received cmd
 *
 * cmd: The command/register address received
 * */
void heater_proccess();
void I2C_Slave_ProcessCMD(unsigned char *message, uint16_t length);
void initGPIO();
void set_adc_channel(int channel);
void initClockTo16MHz();
void initADC();


#endif
