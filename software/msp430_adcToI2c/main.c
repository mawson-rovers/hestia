//******************************************************************************
//   MSP430x261x ADC to I2C
//
//   Description: read from a given adc pin and allow reading over i2c
//   i2c commands can be sent to change the adc pin and turn on the heater
//******************************************************************************

#include <msp430.h>
#if defined(__GNUC__)
#include <in430.h>
#endif
#include <stdint.h>
#include <stdbool.h>
#include "main.h"
#include "i2c.h"

union I2C_Packet_t message_tx;

void transmit_uint(unsigned int value);

volatile unsigned int adc_readings[8];
unsigned int control_sensor = 0; // sensor used for PWM control
double target_temperature = -999;
unsigned int heater_mode = HEATER_MODE_OFF;
unsigned int current_pwm = HEATER_PWM_FREQ_DEFAULT; // Currently bit-banged 8 bit resolution
unsigned int counter = 0;


//******************************************************************************
// Main ************************************************************************
// Enters LPM0 and waits for I2C interrupts. The data sent from the master is  *
// then interpreted and the device will respond accordingly                    *
//******************************************************************************


int main(void) {
    WDTCTL = WDTPW | WDTHOLD;   // Stop watchdog timer
    message_tx.data = ADC_UNKNOWN_VALUE; // init to impossible/hard to reach value for fault detection

    initClockTo16MHz();
    initGPIO();
    initI2C();
    initADC();

//    PRxData = (unsigned char *)RxBuffer;    // Start of RX buffer

    // #TODO continuously read and filter ADC values and send to internal array
    for (;;) {
        //TODO replace with timer
        ADC12CTL0 |= ADC12SC;                   // Start conversion, software controlled
        __bis_SR_register(CPUOFF + GIE + LPM0_bits);        // LPM0, ADC12_ISR will force exit
        heater_proccess();
    }
}

void process_cmd_tx(unsigned char cmd) {
    if (cmd >= COMMAND_SENSOR_LOW && cmd <= COMMAND_SENSOR_HIGH) {
        // set active adc to read from
        unsigned int sensor = cmd - 1;
        transmit_uint(adc_readings[sensor]);
    } else if (cmd == COMMAND_READ_PWM_FREQ) {
        transmit_uint(current_pwm);
    } else if (cmd == COMMAND_READ_HEATER_MODE) {
        transmit_uint(heater_mode);
    } else {
        // Unknown command
    }
}

inline void transmit_uint(unsigned int value) {
    // Fill out the transmit buffer
    message_tx.data = value;
    TransmitLen = 2;
    CopyArray(message_tx.I2CPacket);
}

void I2C_Slave_ProcessCMD(unsigned char *message_rx, uint16_t length) {
    // make more like a read write register thing
    // need to take multiple byte sin first byte is register remaining byte is command
    // http://nilhcem.com/android-things/arduino-as-an-i2c-slave
    uint8_t cmd = message_rx[0];
    unsigned char *package = message_rx + 1; // ignore the command

    if (cmd == COMMAND_WRITE_HEATER_MODE) {
        // Set the heater mode
        heater_mode = package[0];
    } else if (cmd == COMMAND_TARGET_TEMP) {
        target_temperature = *((float *) package); //#TODO check this works
        // Should be as the memory is fully allocated
        TransmitLen = 0;
    } else if (cmd == COMMAND_TARGET_SENSOR) {
        control_sensor = package[0];
        TransmitLen = 0;
    } else if (cmd == COMMAND_WRITE_PWM_FREQ) {
        current_pwm = package[0];
        TransmitLen = 0;
    } else if (cmd == COMMAND_RESET) {
        // TODO implement reset
    } else {
        // unknown command
    }

    // TOOD checksum??
}

void heater_proccess() {
    if (heater_mode == HEATER_MODE_PWM) {
        // TODO PWM currently dosen't seem to be working so bit banging
        // CCR2 = current_pwm;                                 // CCR2 PWM duty cycle 0%
        if (current_pwm > counter) {
            P1OUT |= HEATER_PIN;
            P5OUT |= LED_YELLOW;    // LED_2 on
        } else {
            P1OUT &= ~HEATER_PIN;
            P5OUT &= ~LED_YELLOW;   // LED_2 off
        }
        counter++;
        if (counter > 255) {
            counter = 0;
        }
    } else {
        // just turn everything off
        P1OUT &= ~HEATER_PIN;
        P5OUT &= ~LED_YELLOW;   // LED_2 off
    }
}

void initClockTo16MHz() {
    if (CALBC1_16MHZ == 0xFF)                  // If calibration constant erased
    {
        while (1);                               // do not load, trap CPU!!
    }
    DCOCTL = 0;                               // Select lowest DCOx and MODx settings
    BCSCTL1 = CALBC1_16MHZ;                    // Set DCO
    DCOCTL = CALDCO_16MHZ;
}

void initGPIO() {
    //I2C Pins
    P3SEL |= BIT1 | BIT2;                     // P3.1,2 option select
    P5DIR |= 0x0F;                            // Set P1.0 to output direction
    P5OUT &= ~(LED_YELLOW | LED_GREEN);       // Turn off red led
    P1DIR |= HEATER_PIN;                      // P1.7 output
//    P1SEL |= HEATER_PIN;                      // P1.7 TA2 options
    P1OUT &= ~HEATER_PIN;                    // Turn off the heater
//    CCR0 = 512-1;                             // PWM Period
//    CCTL2 = OUTMOD_7;                         // CCR2 reset/set
//    CCR2 = 100;                                 // CCR2 PWM duty cycle 0%
//    TACTL = TASSEL_2 + MC_1;                  // SMCLK, up mode
}


void initADC() {
    P6SEL = 0x0F;                             // Enable A/D channel inputs
    ADC12CTL0 = ADC12ON + MSC + SHT0_8;           // Turn on ADC12, extend sampling time
    // to avoid overflow of results
    ADC12CTL1 = SHP + CONSEQ_3;                 // Use sampling timer, repeated sequence
    ADC12MCTL0 = INCH_0;                      // ref+=AVcc, channel = A0
    ADC12MCTL1 = INCH_1;                      // ref+=AVcc, channel = A1
    ADC12MCTL2 = INCH_2;                      // ref+=AVcc, channel = A2
    ADC12MCTL3 = INCH_3;                      // ref+=AVcc, channel = A3
    ADC12MCTL4 = INCH_4;                      // ref+=AVcc, channel = A4
    ADC12MCTL5 = INCH_5;                      // ref+=AVcc, channel = A5
    ADC12MCTL6 = INCH_6;                      // ref+=AVcc, channel = A6
    ADC12MCTL7 = INCH_7 + EOS;                  // ref+=AVcc, channel = A7, end seq.
    ADC12IE = 0x08;                           // Enable ADC12IFG.3
    ADC12CTL0 |= ENC;                         // Enable conversions
}

// ADC12 interrupt service routine
#if defined(__TI_COMPILER_VERSION__) || defined(__IAR_SYSTEMS_ICC__)
#pragma vector=ADC12_VECTOR
__interrupt void ADC12_ISR (void)
#elif defined(__GNUC__)
void __attribute__ ((interrupt(ADC12_VECTOR))) ADC12_ISR (void)
#else
#error Compiler not supported!
#endif
{
    adc_readings[0] = ADC12MEM0 >= ADC_MIN_VALUE ? ADC12MEM0 : ADC_UNKNOWN_VALUE;
    adc_readings[1] = ADC12MEM1 >= ADC_MIN_VALUE ? ADC12MEM1 : ADC_UNKNOWN_VALUE;
    adc_readings[2] = ADC12MEM2 >= ADC_MIN_VALUE ? ADC12MEM2 : ADC_UNKNOWN_VALUE;
    adc_readings[3] = ADC12MEM3 >= ADC_MIN_VALUE ? ADC12MEM3 : ADC_UNKNOWN_VALUE;
    adc_readings[4] = ADC12MEM4 >= ADC_MIN_VALUE ? ADC12MEM4 : ADC_UNKNOWN_VALUE;
    adc_readings[5] = ADC12MEM5 >= ADC_MIN_VALUE ? ADC12MEM5 : ADC_UNKNOWN_VALUE;
    adc_readings[6] = ADC12MEM6 >= ADC_MIN_VALUE ? ADC12MEM6 : ADC_UNKNOWN_VALUE;
    adc_readings[7] = ADC12MEM7 >= ADC_MIN_VALUE ? ADC12MEM7 : ADC_UNKNOWN_VALUE;
    // IFG is cleared by reads

    __bic_SR_register_on_exit(CPUOFF);      // Clear CPUOFF bit from 0(SR)
}
