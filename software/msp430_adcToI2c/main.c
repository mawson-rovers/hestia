//******************************************************************************
//   MSP430x261x ADC to I2C
//
//   Description: read from a given adc pin and allow reading over i2c
//   i2c commands can be sent to change the adc pin and turn on the heater #TODO
//******************************************************************************

#include <msp430.h> 
#include <stdint.h>
#include <stdbool.h>
#include "main.h"
#include "i2c.h"

union I2C_Packet_t message;
volatile unsigned int aResults[8];
int acitve_sensor = 0;

//******************************************************************************
// Main ************************************************************************
// Enters LPM0 and waits for I2C interrupts. The data sent from the master is  *
// then interpreted and the device will respond accordingly                    *
//******************************************************************************


int main(void) {
    WDTCTL = WDTPW | WDTHOLD;   // Stop watchdog timer
    message.sensor = 0xFF; // init to bad value

    initClockTo16MHz();
    initGPIO();
    initI2C();
    initADC();

    // #TODO continuously read and filter ADC values and send to internal array
    for (;;)
    {
      //TODO replace with timer
      ADC12CTL0 |= ADC12SC;                   // Start conversion, software controlled
      __bis_SR_register(CPUOFF + GIE + LPM0_bits);        // LPM0, ADC12_ISR will force exit
    }
}

void I2C_Slave_ProcessCMD(uint8_t cmd)
{
    if(cmd<7){
        acitve_sensor = cmd;
        if(cmd>2){
            P5OUT |= LED_YELLOW;                        // LED_1 on
            P5OUT &= ~LED_GREEN;                        // LED_2 off
        }else{
            P5OUT |= LED_GREEN;                          // LED_1 on
            P5OUT &= ~LED_YELLOW;                        // LED_2 off
        }
    }
    // TOOD turn on heater with comand
    // TOOD checksum??
    // Fill out the TransmitBuffer
    CopyArray(message.I2CPacket);
}

void initClockTo16MHz()
{
    if (CALBC1_16MHZ==0xFF)                  // If calibration constant erased
    {
        while(1);                               // do not load, trap CPU!!
    }
    DCOCTL = 0;                               // Select lowest DCOx and MODx settings
    BCSCTL1 = CALBC1_16MHZ;                    // Set DCO
    DCOCTL = CALDCO_16MHZ;
}

void initGPIO()
{
    //I2C Pins
    P3SEL |= BIT1 | BIT2;                     // P3.1,2 option select
    P5DIR |= 0x0F;                            // Set P1.0 to output direction
    P5OUT &= ~(LED_YELLOW | LED_GREEN);                          // Turn off red led
}



void initADC()
{
    P6SEL = 0x0F;                             // Enable A/D channel inputs
    ADC12CTL0 = ADC12ON+MSC+SHT0_8;           // Turn on ADC12, extend sampling time
                                              // to avoid overflow of results
    ADC12CTL1 = SHP+CONSEQ_3;                 // Use sampling timer, repeated sequence
    ADC12MCTL0 = INCH_0;                      // ref+=AVcc, channel = A0
    ADC12MCTL1 = INCH_1;                      // ref+=AVcc, channel = A1
    ADC12MCTL2 = INCH_2;                      // ref+=AVcc, channel = A2
    ADC12MCTL3 = INCH_3;                      // ref+=AVcc, channel = A3
    ADC12MCTL4 = INCH_4;                      // ref+=AVcc, channel = A4
    ADC12MCTL5 = INCH_5;                      // ref+=AVcc, channel = A5
    ADC12MCTL6 = INCH_6;                      // ref+=AVcc, channel = A6
    ADC12MCTL7 = INCH_7+EOS;                  // ref+=AVcc, channel = A7, end seq.
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
    // TODO apply low pass filter
    aResults[0] = ADC12MEM0;             // Move A0 results, IFG is cleared
    aResults[1] = ADC12MEM1;             // Move A1 results, IFG is cleared
    aResults[2] = ADC12MEM2;             // Move A2 results, IFG is cleared
    aResults[3] = ADC12MEM3;             // Move A3 results, IFG is cleared
    aResults[4] = ADC12MEM4;             // Move A4 results, IFG is cleared
    aResults[5] = ADC12MEM5;             // Move A5 results, IFG is cleared
    aResults[6] = ADC12MEM6;             // Move A6 results, IFG is cleared
    aResults[7] = ADC12MEM7;             // Move A7 results, IFG is cleared

    message.sensor = aResults[acitve_sensor]; // update the sensor message not sure if this is a good idea to do here
    __bic_SR_register_on_exit(CPUOFF);      // Clear CPUOFF bit from 0(SR)
}
