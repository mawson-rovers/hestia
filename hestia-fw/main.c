//******************************************************************************
//   Hestia firmware for MSP430F2618.
//
//   I2C interface and control logic for the Hestia circuit board.
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

volatile unsigned int adc_readings[ADC_SENSOR_COUNT];
unsigned int control_sensor = 0; // ADC input used for PWM control
unsigned int set_point = TEMP_0C;
unsigned int heater_mode = HEATER_MODE_OFF;
unsigned int current_pwm = HEATER_PWM_FREQ_DEFAULT; // Currently bit-banged 8 bit resolution
unsigned int counter = 0;

// PID control variables
#define K_P         7       // PID proportional gain
#define K_I_SHIFT   7       // PID integral shift right bits
#define MAX_OUT     1000
#define MIN_OUT     0

int error_sum = 0;


int main(void) {
    WDTCTL = WDTPW | WDTHOLD;   // Stop watchdog timer
    message_tx.data = ADC_UNKNOWN_VALUE; // init to impossible/hard to reach value for fault detection

    initClockTo16MHz();
    initGPIO();
    initI2C();
    initADC();
    initTimer();

    // #TODO continuously read and filter ADC values and send to internal array
    for (;;) {
        //TODO replace with timer
        ADC12CTL0 |= ADC12SC;                   // Start conversion, software controlled
        __bis_SR_register(CPUOFF + GIE + LPM0_bits);        // LPM0, ADC12_ISR will force exit
        heater_process();
    }
}

void initTimer() {
    // Configure Timer A at 250 Hz
    BCSCTL2 |= DIVS_3;                  // SMCLK: 16MHz DCO divided by 8 = 2 MHz (SLAU144K, table 5-4)
    CCR0 = 1000;                        // timer frequency: SMCLK 2 MHz / 1000 = 2 kHz
    TACCTL0 = CCIE;                     // enable A0 interrupt on CCR0 overflow
    CCR2 = 0;                           // duty cycle: CCR2 / 1000
    TACCTL2 = OUTMOD_7;                 // CCR2 reset/set mode for output
    TACTL = TASSEL_2 + MC_1 + ID_3;     // SMCLK, CCR0 up mode, input divider /8 = 250 Hz
}

inline unsigned int update_pid(unsigned int value) {
    int error = (int) set_point - (int) value; // both inputs must be positive and <=2^12 (ADC values)
    error_sum += error;
    int out = K_P * error + (error_sum >> K_I_SHIFT); // MSP430 ABI maintains sign in arithmetic right-shift
    if (out > MAX_OUT) out = MAX_OUT;
    else if (out < MIN_OUT) out = MIN_OUT;
    return (unsigned int) out; // max and min ensure conversion is safe: 0 < out < 2^15
}

unsigned int ta_count = 0;

// Timer A interrupt handler
#if defined(__TI_COMPILER_VERSION__) || defined(__IAR_SYSTEMS_ICC__)
#pragma vector=TIMERA0_VECTOR
__interrupt void Timer_A(void)
#elif defined(__GNUC__)
void __attribute__ ((interrupt(TIMERA0_VECTOR))) Timer_A(void)
#else
#error Compiler not supported!
#endif
{
    ta_count++;
    if (ta_count > 250) {
        ta_count = 0;
        if (heater_mode == HEATER_MODE_PID) {
            P5OUT ^= LED_YELLOW;     // toggle PID indicator LED
            unsigned int adc_value = adc_readings[control_sensor];
            if (adc_value >= ADC_MIN_VALUE && adc_value <= ADC_MAX_VALUE) {
                CCR2 = update_pid(adc_value);
            } else {
                CCR2 = 0;
            }
        } else {
            CCR2 = 0;
            P5OUT &= ~(LED_YELLOW); // turn off PID indicator LED
        }
    }
}

void process_cmd_tx(unsigned char cmd) {
    if (cmd >= COMMAND_READ_SENSOR_LOW && cmd <= COMMAND_READ_SENSOR_HIGH) {
        // set active adc to read from
        unsigned int sensor = cmd - 1;
        transmit_uint(adc_readings[sensor]);
    } else if (cmd == COMMAND_READ_BOARD_VERSION) {
        transmit_uint(HESTIA_VERSION);
    } else if (cmd == COMMAND_READ_HEATER_MODE) {
        transmit_uint(heater_mode);
    } else if (cmd == COMMAND_READ_TARGET_TEMP) {
        transmit_uint(set_point);
    } else if (cmd == COMMAND_READ_TARGET_SENSOR) {
        transmit_uint(control_sensor);
    } else if (cmd == COMMAND_READ_PWM_FREQ) {
        if (heater_mode == HEATER_MODE_PID) {
            transmit_uint(CCR2);
        } else {
            transmit_uint(current_pwm);
        }
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

    P5OUT ^= (HESTIA_VERSION < 200) ? LED_GREEN : LED_BLUE;

    if (cmd == COMMAND_WRITE_HEATER_MODE) {
        // Set the heater mode
        heater_mode = package[0];
        if (heater_mode == HEATER_MODE_PWM) {
            P1SEL &= ~HEATER_PIN;                     // P1.7 disable TA2 option
        } else if (heater_mode == HEATER_MODE_PID) {
            P1SEL |= HEATER_PIN;                      // P1.7 enable TA2 option
        }
    } else if (cmd == COMMAND_WRITE_TARGET_TEMP) {
        if (length >= 2) {
            set_point = (package[1] << 8) + package[0];
        }
        TransmitLen = 0;
    } else if (cmd == COMMAND_WRITE_TARGET_SENSOR) {
        if (package[0] < ADC_SENSOR_COUNT) {
            control_sensor = package[0];
        }
        TransmitLen = 0;
    } else if (cmd == COMMAND_WRITE_PWM_FREQ) {
        current_pwm = package[0];
        TransmitLen = 0;
    } else if (cmd == COMMAND_RESET) {
        WDTCTL = 0xDEAD;  // write to the WDT password to trigger a reset
    } else {
        // unknown command
    }
}

void heater_process() {
    if (heater_mode == HEATER_MODE_PWM) {
        // TODO PWM currently doesn't seem to be working so bit banging
        // CCR2 = current_pwm;                                 // CCR2 PWM duty cycle 0%
        if (current_pwm > counter) {
            P1OUT |= HEATER_PIN;
            if (HESTIA_VERSION < 200) P5OUT |= LED_YELLOW;    // LED on
        } else {
            P1OUT &= ~HEATER_PIN;
            if (HESTIA_VERSION < 200) P5OUT &= ~LED_YELLOW;   // LED off
        }
        counter++;
        if (counter > 255) {
            counter = 0;
        }
    } else if (heater_mode == HEATER_MODE_PID) {
        // do nothing - PID timer will take care of heater pin and LEDs
    } else {
        // turn everything off
        P1OUT &= ~HEATER_PIN;
        if (HESTIA_VERSION < 200) P5OUT &= ~LED_YELLOW;   // LED off
    }
}

void initClockTo16MHz() {
    if (CALBC1_16MHZ == 0xFF) {
        while (1); // trap CPU if calibration constant not found
    }
    DCOCTL = 0;                 // Select lowest DCOx and MODx settings
    BCSCTL1 = CALBC1_16MHZ;     // Set DCO to 16 MHz
    DCOCTL = CALDCO_16MHZ;
}

void initGPIO() {
    // I2C Pins
    P3SEL |= BIT1 | BIT2;                     // P3.1,2 for I2C

    // Status LEDs
    P5DIR |= (LED_YELLOW | LED_GREEN | LED_BLUE);  // LED output pins
    P5OUT &= ~(LED_YELLOW | LED_GREEN | LED_BLUE); // Turn off status LEDs

    P1DIR |= HEATER_PIN;                      // P1.7 is output
    P1OUT &= ~HEATER_PIN;                     // Set heater off
    P1SEL |= HEATER_PIN;                      // P1.7 TA2 option
}

void initADC() {
    P6SEL = 0x0F;                             // Enable A/D channel inputs
    ADC12CTL0 = ADC12ON + MSC;                // Turn on ADC12, multiple sample/conv mode
    ADC12CTL0 |= SHT0_8;                      // Sample+hold time: 256 ADC12CLK cycles (~19.5 KHz)
    ADC12CTL1 = SHP + CONSEQ_3;               // Use sampling timer, repeated sequence
    ADC12CTL1 |= ADC12SSEL_0;                 // Use ADC12OSC internal oscillator (~5 MHz)
    ADC12CTL1 |= ADC12DIV_0;                  // ADC12 clock divider = /1
    ADC12MCTL0 = INCH_0;                      // ref+=AVcc, channel = A0
    ADC12MCTL1 = INCH_1;                      // ref+=AVcc, channel = A1
    ADC12MCTL2 = INCH_2;                      // ref+=AVcc, channel = A2
    ADC12MCTL3 = INCH_3;                      // ref+=AVcc, channel = A3
    ADC12MCTL4 = INCH_4;                      // ref+=AVcc, channel = A4
    ADC12MCTL5 = INCH_5;                      // ref+=AVcc, channel = A5
    ADC12MCTL6 = INCH_6;                      // ref+=AVcc, channel = A6
    ADC12MCTL7 = INCH_7 + EOS;                // ref+=AVcc, channel = A7, end seq.
    ADC12IE = 0x80;                           // Enable ADC12IFG.7
    ADC12CTL0 |= ENC;                         // Enable conversions
}

// ADC12 interrupt service routine
#if defined(__TI_COMPILER_VERSION__) || defined(__IAR_SYSTEMS_ICC__)
#pragma vector=ADC12_VECTOR
__interrupt void ADC12_ISR(void)
#elif defined(__GNUC__)
void __attribute__ ((interrupt(ADC12_VECTOR))) ADC12_ISR(void)
#else
#error Compiler not supported!
#endif
{
    adc_readings[0] = ADC12MEM0;
    adc_readings[1] = ADC12MEM1;
    adc_readings[2] = ADC12MEM2;
    adc_readings[3] = ADC12MEM3;
    adc_readings[4] = ADC12MEM4;
    adc_readings[5] = ADC12MEM5;
    adc_readings[6] = ADC12MEM6;
    adc_readings[7] = ADC12MEM7;
    // IFG is cleared by reads

    __bic_SR_register_on_exit(CPUOFF);      // Clear CPUOFF bit from 0(SR)
}
