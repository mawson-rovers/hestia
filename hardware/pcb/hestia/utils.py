
# https://www.youtube.com/watch?v=_zVJ96SdYrs
# https://forum.kicad.info/t/python-scripting-example-studio-clock/5387
# http://www.jeffmcbride.net/2019/05/28/programatic-layout-with-kicad-and-python.html
# https://kicad.mmccoo.com/2017/02/22/how-to-add-mounting-holes/

import pcbnew
import numpy as np

#Some Constants for better Readability
LayerFCu = 0
LayerBCu = 31
LayerEdgeCuts = 44

# Converts mm to PCB internal units
SCALE = 1000000

def find_layer(board, layer_name):
    for i in range(128):
        if board.GetLayerName(i) == layer_name:
            return i
    return -1


def delete_tracks(pcb, name=None):
    tracks = pcb.GetTracks()
    if(len(tracks) > 0):
        print('---------------------------------------------------------------')
        print('---Delete-Nets-------------------------------------------------')
        print('---------------------------------------------------------------')

        for t in tracks:
            pcb.Delete(t)
        # segments = pcb.GetSegments() # command dose not exist
        # for seg in segments:
        # 	pcb.Delete(seg)
    else:
        print("No existring tracks")
    

def np_to_point(pos):
    return pcbnew.wxPointMM(float(pos[0]), float(pos[1]))



def addtracks(pcb, array, width=0.5, layer=LayerFCu):
    # given an array of keypoints lay track between them
    len = array.shape[0]-1
    for i in range(len):
        vec = array[i:i+2] # extract the vector between to keypoints
        addtrack(pcb, vec, width=width, layer=layer)

def addtrack(pcb, vec, width=0.5, layer=LayerFCu):
    """ add a track """
    vec = vec.astype(dtype=float)
    t = pcbnew.PCB_TRACK(pcb)
    t.SetStart(pcbnew.wxPointMM(float(vec[0,0]), float(vec[0,1])))
    t.SetEnd(pcbnew.wxPointMM(float(vec[1,0]), float(vec[1,1])))
    t.SetNetCode(0)
    t.SetLayer(layer)
    t.SetWidth(int(width*SCALE))
    pcb.Add(t)

def square_outline(pcb, corners = [[-1,-1],[-1,1],[1,1],[1,-1]]):
    print("Setting Board Dimensions too:")
    l = 10*1.2
    for i in range(4):
        seg = pcbnew.PCB_SHAPE(pcb)
        pcb.Add(seg)
        seg.SetStart(pcbnew.wxPointMM(corners[i][0]*l, corners[i][1]*l))
        seg.SetEnd(pcbnew.wxPointMM(corners[(i+1)%4][0]*l, corners[(i+1)%4][1]*l))
        seg.SetLayer(LayerEdgeCuts)
        print("Corner:", seg.GetStart())

def calculate_trace_length(trace):
    length = 0
    for i in range(trace.shape[0]-1):
        dis = trace[i]-trace[i+1]
        length += np.hypot(dis[0], dis[1])
    return length


def plot_trace(trace):
    import matplotlib.pyplot as plt
    plt.plot(trace[:,0], trace[:,1])
    plt.gca().invert_yaxis() # try to look like kicad
    plt.gca().set_aspect('equal')
    plt.grid()
    plt.show()
