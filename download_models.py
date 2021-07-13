import os
import wget
import zipfile
import shutil
import tempfile

from app.config import config

from app.tasks.spacy_pipe import get_spacy_path, spacy_model

def download(url, path):
    file = wget.download(url, path)
    return file


def extract(filepath):
    path, _ = os.path.split(filepath)
    with zipfile.ZipFile(filepath) as zip_ref:
        zip_ref.extractall(path)


def download_all_models():
    download_vosk_models()
    download_spacy_models()


def download_spacy_models():
    # spacy models are python pip packages.
    # this downloads the model to a tempdir, and then copies the path
    # to storage_path/models/spacy. this path is added to sys.path in the
    # spacy constructor at runtime.

    spacy_path = get_spacy_path()
    os.makedirs(spacy_path, exist_ok=True)
    pip_options = f'--prefix="{spacy_path}"'
    command = f'python -m spacy download {spacy_model} {pip_options}'
    os.system(command)
    #  spa
    #  ls = os.listdir(os.path.join(tempdir, 'lib'))
    #  #  path = os.path.join(tempdir, f'lib/{ls[0]}/site-packages')
    #  path = tempdir
    #  target = os.path.join(config.storage_path, 'models', 'spacy')
    #  shutil.copytree(path, target)
    #  shutil.rmtree(tempdir)


def download_vosk_models():
    models = {
        "vosk-model-de-0.6": "https://alphacephei.com/vosk/models/vosk-model-de-0.6.zip",
        "vosk-model-spk-0.4": "https://alphacephei.com/vosk/models/vosk-model-spk-0.4.zip"
    }

    models_path = os.path.join(config.storage_path, "models")
    if not os.path.isdir(models_path):
        os.makedirs(models_path)

    for model in models:
        target_dir = os.path.join(models_path, model)
        if not os.path.isdir(target_dir):
            print(f'Downloading {models[model]}')
            filepath = download(models[model], models_path)
            extract(filepath)
        else:
            print(f'Skipping {models[model]}')


if __name__ == "__main__":
    download_all_models()
