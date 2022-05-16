##!/usr/bin/python3
"""
Generates the heater tracks required for the PCB heater
"""

"""
within kicad python terminal in the pcb view 
cd my_stuff/hestia/hardware/pcb/hestia
import os, sys
sys.path.append(os.getcwd())
import heater_track_gen
import importlib
def gen():
    importlib.reload(heater_track_gen)
"""

# https://www.youtube.com/watch?v=_zVJ96SdYrs
# https://forum.kicad.info/t/python-scripting-example-studio-clock/5387
# http://www.jeffmcbride.net/2019/05/28/programatic-layout-with-kicad-and-python.html
# https://kicad.mmccoo.com/2017/02/22/how-to-add-mounting-holes/


import numpy as np

import utils

angs = np.array([[1,0],
        [0,1],
        [-1,0],
        [0,-1]])

base_curve = np.array([[0,1],
                [1,1],
                [1,0],
                [0,0]])
base_curve = base_curve-np.array([0.5,0.5])

# precompute rotation matrixs for speed?
rotation_matrix = np.zeros((4,2,2), dtype=int)
for i in range(4):
    ang = 2*np.pi*i/4
    rotation_matrix[i] = [  [np.cos(ang),-np.sin(ang)],
                            [np.sin(ang), np.cos(ang)]]

def rotate(angle):
    """ rotate a path where angle is int 1-3 correspoding to degrees 0-270 """
    new_curve = np.zeros_like(base_curve)
    for i in range(4):
        new_curve[i] = base_curve[i].dot(rotation_matrix[angle])
    return new_curve

def add_base_curve(loc, angle, size):
    """ a new base hilbert curve as specified angle and size """
    angle = angle%4
    new_curve = rotate(angle)
    new_curve *= size
    new_curve += loc       
    return new_curve

def hilbert_curve(path, pos, depth, level, angle, flip=False):
    """
    hilbert curve heater trace
    why hilbert no reason just looks like fun also supposedly less fatigue if bent (only a concern for flex) also thought it might be easer to add in holes
    https://electronics.stackexchange.com/questions/602319/using-a-pcb-trace-as-a-heater-hilbert-curves
    https://www.geeksforgeeks.org/python-hilbert-curve-using-turtle/
    https://warehouse-camo.ingress.cmh1.psfhosted.org/10745f1d11eb77724a4ab8721159be851675b7e7/68747470733a2f2f7261772e67697468756275736572636f6e74656e742e636f6d2f67616c7461792f68696c6265727463757276652f6d61696e2f6e325f70332e706e67
    """

    new_curve = add_base_curve(pos, angle, 1/(2**depth))
    if(flip):
        new_curve = np.flip(new_curve, axis=0)
    if(level==1):
        if(path is None):
            return new_curve
        else:
            return np.vstack((path,new_curve))
    if(level<1):
        assert(), f"hilbert level {level} <0"

    # arg ugly hack with the flip #TODO clean up
    path = hilbert_curve(path, new_curve[0], depth+1, level-1, angle+1-2*flip, flip=not flip)
    path = hilbert_curve(path, new_curve[1], depth+1, level-1, angle, flip = flip)
    path = hilbert_curve(path, new_curve[2], depth+1, level-1, angle, flip = flip)
    path = hilbert_curve(path, new_curve[3], depth+1, level-1, angle-1-2*flip, flip = not flip)

    return path

def hilbert_curve_length(depth, size = 10):
    """
    calculate the depth of a given hilbert curve 
    from https://math.stackexchange.com/questions/2034027/length-of-hilbert-curve-in-3-dimensions
    """
    length =  2**depth - (1/(2**depth))
    return length*size

def gen_heater_trace(hilbert_depth=4, size=10, center=[0,0]):
    """ generate heater trace """
    path = None
    pos = np.array([0,0])
    path = hilbert_curve(path, pos, 1, hilbert_depth, 3, flip=False)
    path*=size
    path += center
    return path

def resistance_per_mm(width, copper_weight=1, temperature=25):
    """
    calcualte track resistance
    https://www.allaboutcircuits.com/tools/trace-resistance-calculator/
    param:
    width in mm
    copper weight in oz/ft
    """
    height = 0.0347*copper_weight # convert from oz/ft to mm height
    ρcopper  =  1.7e-5 #ohm/mm
    acopper = 3.9e-3 #ohm/ohm/C
    R = ρcopper * (1/(width * height)) * (1 + acopper * (temperature-25))
    return R

def calc_width(length, R):
    """ given a length and desired resistance calculate needed track length """
    r_mm_desired = R/length
    r_mm = resistance_per_mm(1)
    width_desired = r_mm/r_mm_desired
    return width_desired

def watts2R(watts, V=5):
    """ given a desired wattage and voltage calculate need resistance """
    I = watts/V
    #V=IR
    R = V/I
    return R

if __name__ == "__main__":
    # for a given hilber curve calulate the feature size (space witbetween middle of tracks)
    # take 50% and use a strack widht thern cauclate ideal resitance per deoth
    # R = resistance_per_mm(1)*1000
    desired_R = watts2R(15)
    print(f"desired R:{desired_R:.3f}")
    length = hilbert_curve_length(5, size=30)
    print(f"length:{length:.3f}")
    width = calc_width(length, desired_R)
    print(f"width:{width:.5f}\tminmum:0.127mm")
    # print(f"resistance:{R}")
    exit()
    # simple ploting if run standalone
    depths = np.arange(1,10,dtype=int)
    lengths = np.zeros_like(depths)
    for i, depth in enumerate(depths):
        # trace = gen_heater_trace(hilbert_depth=depth)
        # lengths[i] = utils.calculate_trace_length(trace)
        lengths[i] = hilbert_curve_length(depth)
    import matplotlib.pyplot as plt
    
    plt.plot(depths, lengths)
    plt.plot(depths, np.load("tmp_og.npy"))
    plt.show()
    # print(f"trace length = {length:.2f}mm")
    # print("estimated resistance = ??")
    # utils.plot_trace(trace)

else:
    # if not main assume run from kicad and generate traces
    import pcbnew
    import importlib
    importlib.reload(utils)
    pcb = pcbnew.GetBoard()
    # io = pcbnew.io now ? FootprintLoad()

    footprint_path = 'kicad-footprints/'
    #Some Constants for better Readability
    LayerFCu = 0
    LayerBCu = 31
    LayerEdgeCuts = 44
    LayerFSilk = utils.find_layer(pcb, "F.SilkS")


    utils.delete_tracks(pcb)

    layertable = {}

    # for i in range(1000):
    #     name = pcb.GetLayerName(i)
    #     if name != "BAD INDEX!":
    #         layertable[name]=i
    #print(layertable)
    
    center = np.asarray([90.17,-95.885])/2 # board dimensions /2
    center += np.asarray([101.6, 157]) # reference offset
    center += np.asarray([0,5]) #shift down away from connector
    width = 0.28
    V = 5
    print(center)
    trace = gen_heater_trace(hilbert_depth=5, size=30, center=center)#, center=center)
    length = utils.calculate_trace_length(trace)
    R_mm =resistance_per_mm(width)
    R = R_mm*length
    I = (V/R)
    W = I*V
    print(f"Watts at 5V:{W:.2f}\tResistance:{R:.4f}ohms\tlength:{length:.2f}:mm")
    utils.addtracks(pcb, trace, width=width)


    # pcb.Save(f'heater_block_V00_{i}_{d:.1f}mm.kicad_pcb')
    pcbnew.Refresh()