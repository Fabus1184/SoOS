"""
Convert images to vga mode 13h representation
"""
import sys
from PIL import Image
from math import sqrt
from functools import reduce
from operator import concat
from typing import Iterable, cast, TypeVar

T = TypeVar('T')

COLORS: dict[str, int] = {
    '000000': 0, '0000aa': 1, '00aa00': 2, '00aaaa': 3, 'aa0000': 4, 'aa00aa': 5, 'aa5500': 6, 'aaaaaa': 26, '555555': 21, '5555ff': 9, '55ff55': 10,
    '55ffff': 11, 'ff5555': 12, 'ff55ff': 13, 'ffff55': 14, 'ffffff': 31, '101010': 17, '202020': 18, '353535': 19, '454545': 20, '656565': 22, '757575': 23,
    '8a8a8a': 24, '9a9a9a': 25, 'bababa': 27, 'cacaca': 28, 'dfdfdf': 29, 'efefef': 30, '0000ff': 32, '4100ff': 33, '8200ff': 34, 'be00ff': 35, 'ff00ff': 36,
    'ff00be': 37, 'ff0082': 38, 'ff0041': 39, 'ff0000': 40, 'ff4100': 41, 'ff8200': 42, 'ffbe00': 43, 'ffff00': 44, 'beff00': 45, '82ff00': 46, '41ff00': 47,
    '00ff00': 48, '00ff41': 49, '00ff82': 50, '00ffbe': 51, '00ffff': 52, '00beff': 53, '0082ff': 54, '0041ff': 55, '8282ff': 56, '9e82ff': 57, 'be82ff': 58,
    'df82ff': 59, 'ff82ff': 60, 'ff82df': 61, 'ff82be': 62, 'ff829e': 63, 'ff8282': 64, 'ff9e82': 65, 'ffbe82': 66, 'ffdf82': 67, 'ffff82': 68, 'dfff82': 69,
    'beff82': 70, '9eff82': 71, '82ff82': 72, '82ff9e': 73, '82ffbe': 74, '82ffdf': 75, '82ffff': 76, '82dfff': 77, '82beff': 78, '829eff': 79, 'babaff': 80,
    'cabaff': 81, 'dfbaff': 82, 'efbaff': 83, 'ffbaff': 84, 'ffbaef': 85, 'ffbadf': 86, 'ffbaca': 87, 'ffbaba': 88, 'ffcaba': 89, 'ffdfba': 90, 'ffefba': 91,
    'ffffba': 92, 'efffba': 93, 'dfffba': 94, 'caffba': 95, 'baffba': 96, 'baffca': 97, 'baffdf': 98, 'baffef': 99, 'baffff': 100, 'baefff': 101, 'badfff': 102,
    'bacaff': 103, '000071': 104, '1c0071': 105, '390071': 106, '550071': 107, '710071': 108, '710055': 109, '710039': 110, '71001c': 111, '710000': 112,
    '711c00': 113, '713900': 114, '715500': 115, '717100': 116, '557100': 117, '397100': 118, '1c7100': 119, '007100': 120, '00711c': 121, '007139': 122,
    '007155': 123, '007171': 124, '005571': 125, '003971': 126, '001c71': 127, '393971': 128, '453971': 129, '553971': 130, '613971': 131, '713971': 132,
    '713961': 133, '713955': 134, '713945': 135, '713939': 136, '714539': 137, '715539': 138, '716139': 139, '717139': 140, '617139': 141, '557139': 142,
    '457139': 143, '397139': 144, '397145': 145, '397155': 146, '397161': 147, '397171': 148, '396171': 149, '395571': 150, '394571': 151, '515171': 152,
    '595171': 153, '615171': 154, '695171': 155, '715171': 156, '715169': 157, '715161': 158, '715159': 159, '715151': 160, '715951': 161, '716151': 162,
    '716951': 163, '717151': 164, '697151': 165, '617151': 166, '597151': 167, '517151': 168, '517159': 169, '517161': 170, '517169': 171, '517171': 172,
    '516971': 173, '516171': 174, '515971': 175, '000041': 176, '100041': 177, '200041': 178, '310041': 179, '410041': 180, '410031': 181, '410020': 182,
    '410010': 183, '410000': 184, '411000': 185, '412000': 186, '413100': 187, '414100': 188, '314100': 189, '204100': 190, '104100': 191, '004100': 192,
    '004110': 193, '004120': 194, '004131': 195, '004141': 196, '003141': 197, '002041': 198, '001041': 199, '202041': 200, '282041': 201, '312041': 202,
    '392041': 203, '412041': 204, '412039': 205, '412031': 206, '412028': 207, '412020': 208, '412820': 209, '413120': 210, '413920': 211, '414120': 212,
    '394120': 213, '314120': 214, '284120': 215, '204120': 216, '204128': 217, '204131': 218, '204139': 219, '204141': 220, '203941': 221, '203141': 222,
    '202841': 223, '2d2d41': 224, '312d41': 225, '352d41': 226, '3d2d41': 227, '412d41': 228, '412d3d': 229, '412d35': 230, '412d31': 231, '412d2d': 232,
    '41312d': 233, '41352d': 234, '413d2d': 235, '41412d': 236, '3d412d': 237, '35412d': 238, '31412d': 239, '2d412d': 240, '2d4131': 241, '2d4135': 242,
    '2d413d': 243, '2d4141': 244, '2d3d41': 245, '2d3541': 246, '2d3141': 247
}


def normalize(r_g_b: tuple[float, float, float]) -> tuple[float, float, float]:
    return (lambda m: map(lambda x: x / m, r_g_b) if m >= 0.0001 else (0.0, 0.0, 0.0))(sqrt(sum(map(lambda x: x ** 2, r_g_b))))


def color_distance(c1: tuple[float, float, float], c2: tuple[float, float, float]) -> float:
    return sqrt(sum(map(lambda v1, v2: pow(v1 - v2, 2), c1, c2)))


def nearest_representation(col: tuple[float, float, float]) -> int:
    return COLORS[
        min(COLORS.keys(), key=lambda k: color_distance(col, cast(tuple[float, float, float], map(lambda i: float(int(k[i:i + 2], 16)), range(0, 6, 2)))))]


def plane_coords(x_y: tuple[int, int]) -> list[tuple[int, int]]:
    return cast(list[tuple[int, int]], reduce(concat, cast(Iterable[T], [[(a, b) for a in range(x_y[0])] for b in range(x_y[1])])))


def main() -> None:
    if len(sys.argv) != 2:
        sys.stderr.write(
            "Usage: python convert.py image-file\n\n"
            "where image-file can be any file PIL will recognize\n"
        )
        sys.exit(1)

    else:
        image = Image.open(sys.argv[1]).convert("RGB")
        for x, y in plane_coords(image.size):
            print(nearest_representation(image.getpixel((x, y))), end=",")
            print("") if x == y and {x, y}.isdisjoint({0}) else None
        print("\b")
        print(f"Image size: {image.size}")


if __name__ == "__main__":
    main()
