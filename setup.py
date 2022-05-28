from setuptools import setup, find_packages

VERSION = '0.0.6'
DESCRIPTION = 'My first Python package'
LONG_DESCRIPTION = 'My first Python package with a slightly longer description'

# Setting up
setup(
    name="new_york_calculate",
    version=VERSION,
    author="Andrey Makarov",
    author_email="<vivalaakam@gmail.com>",
    description=DESCRIPTION,
    long_description=LONG_DESCRIPTION,
    packages=find_packages(),
    install_requires=["numpy"],  # add any additional packages that
    # needs to be installed along with your package. Eg: 'caer'

    keywords=['python', 'first package'],
    classifiers=[]
)
