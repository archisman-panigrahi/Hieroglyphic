import os
import random
import tarfile
import json
from shutil import copyfile
from PIL import Image, ImageDraw
from pathlib import Path

# adapted from https://j3698.github.io/extexify/training-the-symbol-recognizer

# Training the model
#
# 1. Run this file. It will generate the traning images,
#    split them into train, val and test sets and combine
#    them into a tar file.
#
# 2. Train the model (see `train.ipynb`) for more details
#
# 3. Replace the model in the data directory
#

FILE_PATH = 'training_data.json'
SIZE = 32

print("Reading training data")
# map from key to list of stroke samples
# each stroke sample is a list of strokes, which is a list of points
symbol_to_stroke_samples = {}

with open(FILE_PATH, 'r') as json_file:
    data = json.load(json_file)
    for symbol_key, symbol_data in data.items():
        symbol_to_stroke_samples[symbol_key] = [[[(point['x'], point['y']) for point in points] for points in value['strokes']] for value in symbol_data]
print(f"Found {len(symbol_to_stroke_samples.keys())} different class")


def draw_image(strokes):
    image = Image.new("L", (SIZE, SIZE), color=0)
    draw = ImageDraw.Draw(image)

    for stroke in strokes:
        # Scale the stroke points to the image size
        scaled_stroke = [(int(x * (SIZE - 1)), int(y * (SIZE - 1))) for x, y in stroke]

        # Draw the stroke by connecting the points
        if len(scaled_stroke) > 1:
            draw.line(scaled_stroke, fill=255, width=1)
        elif len(scaled_stroke) == 1:
            # If the stroke has only one point, draw a point
            draw.point(scaled_stroke, fill=255)

    return image

created_counter = 0
counter = 0

print("Creating images")
for label, strokes_list in symbol_to_stroke_samples.items():
    label_dir = os.path.join(Path().absolute().joinpath(f"images{SIZE}"), label)
    os.makedirs(label_dir, exist_ok=True)

    for strokes in strokes_list:
        image_path = os.path.join(label_dir, f"{counter}.png")
        counter += 1

        if not os.path.exists(image_path):
            image = draw_image(strokes)
            image.save(image_path)
            created_counter += 1
        else:
            print(f"Image {image_path} already exists")

print(f"Created {created_counter} images")


# split data into trainig data
print("Splitting data")

os.makedirs(f"images_data{SIZE}/train", exist_ok = True)
os.makedirs(f"images_data{SIZE}/val", exist_ok = True)
os.makedirs(f"images_data{SIZE}/test", exist_ok = True)

random.seed(0)
files = []
for root, dirs, files in os.walk(f"images{SIZE}"):
    random.shuffle(files)
    for image_file in files:
        _, label = os.path.split(root)
        pick = random.random()

        # ensure the class directory exists
        os.makedirs(f"images_data{SIZE}/train/{label}", exist_ok = True)
        os.makedirs(f"images_data{SIZE}/val/{label}", exist_ok = True)
        os.makedirs(f"images_data{SIZE}/test/{label}", exist_ok = True)
        image_path = os.path.join(root, image_file)

        # make sure at least one of every class is in each split
        if len(os.listdir(f"images_data{SIZE}/train/{label}")) == 0:
            copyfile(image_path, os.path.join(f"images_data{SIZE}/train/{label}", image_file))
        if len(os.listdir(f"images_data{SIZE}/val/{label}")) == 0:
            copyfile(image_path, os.path.join(f"images_data{SIZE}/val/{label}", image_file))
        if len(os.listdir(f"images_data{SIZE}/test/{label}")) == 0:
            copyfile(image_path, os.path.join(f"images_data{SIZE}/test/{label}", image_file))

        # otherwise, choose a split to add the image to
        if pick < 0.7:
            copyfile(image_path, os.path.join(f"images_data{SIZE}/train/{label}", image_file))
        elif pick < 0.9:
            copyfile(image_path, os.path.join(f"images_data{SIZE}/val/{label}", image_file))
        else:
            copyfile(image_path, os.path.join(f"images_data{SIZE}/test/{label}", image_file))

# combine directories in tar file, so it can easily be uploaded for training
with tarfile.open("images.tar.xz", "w:xz") as tar:
    tar.add(f"images_data{SIZE}/train", arcname="train")
    tar.add(f"images_data{SIZE}/val", arcname="val")
    tar.add(f"images_data{SIZE}/test", arcname="test")
