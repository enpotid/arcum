import os
from PIL import Image

# Constants
CLIP_FRAMES = 6571
CLIP_LENGTH = 219.0666
ASCII_CHARS = ['⠀', '⠄', '⠆', '⠖', '⠶', '⡶', '⣩', '⣪', '⣫', '⣾', '⣿'][::-1]
WIDTH = 60
TIMEOUT = 1 / ((CLIP_FRAMES + 1) / CLIP_LENGTH) * 18

def resize(image, new_width=WIDTH):
    (old_width, old_height) = image.size
    aspect_ratio = float(old_height) / float(old_width)
    new_height = int((aspect_ratio * new_width) / 2)
    new_dim = (new_width, new_height)
    new_image = image.resize(new_dim)
    return new_image

def grayscalify(image):
    return image.convert('L')

def modify(image, buckets=25):
    initial_pixels = list(image.getdata())
    new_pixels = [ASCII_CHARS[pixel_value // buckets] for pixel_value in initial_pixels]
    return ''.join(new_pixels)

def do(image, new_width=WIDTH):
    image = resize(image)
    image = grayscalify(image)

    pixels = modify(image)
    len_pixels = len(pixels)

    new_image = [pixels[index:index+int(new_width)] for index in range(0, len_pixels, int(new_width))]

    return '\n'.join(new_image)

def runner(path):
    try:
        image = Image.open(path)
        image = do(image)
        return image
    except Exception as e:
        print(f"Unable to find image in {path}: {e}")
        return None

def save_ascii_frame(ascii_frame, frame_num):
    output_dir = "output"
    if not os.path.exists(output_dir):
        os.makedirs(output_dir)
    
    file_path = os.path.join(output_dir, f"frame{frame_num}.txt")
    with open(file_path, "w", encoding="utf-8") as f:
        f.write(ascii_frame)

if __name__ == '__main__': 
    # Process and save all frames
    for i in range(0, int(CLIP_FRAMES) + 1):
        path = f"frames/frame{i}.jpg"
        ascii_frame = runner(path)
        if ascii_frame:
            save_ascii_frame(ascii_frame, i)
