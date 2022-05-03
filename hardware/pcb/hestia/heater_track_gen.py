##!/usr/bin/python3
"""
Generates the heater tracks required for the PCB heater
"""

"""
within kicad python terminal in the pcb view 
cd hestia/hardware/pcb/hestia
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
    """ rotate a path where angle is int 1-3 correspoding to degrees 0-270"""
    new_curve = np.zeros_like(base_curve)
    for i in range(4):
        new_curve[i] = base_curve[i].dot(rotation_matrix[angle])
    return new_curve

def add_base_curve(loc, angle, size):
    """a new base hilbert curve as specified angle and size """
    angle = angle%4
    new_curve = rotate(angle)
    new_curve *= size
    new_curve += loc       
    return new_curve

def hilbert_2(path, pos, depth, level, angle, flip=False):
    # hilbert curve heater trace
    # why hilbert no reason just looks like fun also supposedly less fatigue if bent (only a concern for flex) also thought it might be easer to add in holes
    # https://electronics.stackexchange.com/questions/602319/using-a-pcb-trace-as-a-heater-hilbert-curves
    # https://www.geeksforgeeks.org/python-hilbert-curve-using-turtle/
    # https://warehouse-camo.ingress.cmh1.psfhosted.org/10745f1d11eb77724a4ab8721159be851675b7e7/68747470733a2f2f7261772e67697468756275736572636f6e74656e742e636f6d2f67616c7461792f68696c6265727463757276652f6d61696e2f6e325f70332e706e67

    new_curve = add_base_curve(pos, angle, 1/(2**depth))
    if(flip):
        new_curve = np.flip(new_curve, axis=0)
    if(level==1):
        if(path is None):
            return new_curve
        else:
            return np.vstack((path,new_curve))

    # arg ugly hack with the flip #TODO clean up
    path = hilbert_2(path, new_curve[0], depth+1, level-1, angle+1-2*flip, flip=not flip)
    path = hilbert_2(path, new_curve[1], depth+1, level-1, angle, flip = flip)
    path = hilbert_2(path, new_curve[2], depth+1, level-1, angle, flip = flip)
    path = hilbert_2(path, new_curve[3], depth+1, level-1, angle-1-2*flip, flip = not flip)

    return path
  

def gen_heater_trace(level, size=10):
    level = 4
    path = None
    pos = np.array([0,0])
    path = hilbert_2(path, pos, 1, level, 3, flip=False)
    path*=size
    return path

if __name__ == "__main__":
    # simple ploting if run standalone
    import matplotlib.pyplot as plt
    trace = gen_heater_trace(2)
    length = utils.calculate_trace_length(trace)
    print(f"trace length = {length:.2f}mm")
    print("estimated resistance = ??")
    exit()
    plt.plot(trace[:,0], trace[:,1])
    # trace = gen_heater_trace(2)
    # plt.plot(trace[:,0], trace[:,1])
    # plt.gca().invert_yaxis()
    plt.gca().set_aspect('equal')
    plt.grid()
    plt.show()

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

    print(layertable)



    # test_track = np.array([[0,0],[100,100]])
    # utils.addtrack(pcb, test_track)

    trace = gen_heater_trace(3, size=50)
    utils.addtracks(pcb, trace)


    # pcb.Save(f'heater_block_V00_{i}_{d:.1f}mm.kicad_pcb')
    pcbnew.Refresh()