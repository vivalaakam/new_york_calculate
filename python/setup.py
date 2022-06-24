from setuptools import setup, find_packages
from setuptools_rust import RustExtension

VERSION = "0.0.26"
DESCRIPTION = "My first Python package"
LONG_DESCRIPTION = "My first Python package with a slightly longer description"

# Setting up
setup(
    name="new_york_calculate",
    version=VERSION,
    author="Andrey Makarov",
    author_email="<vivalaakam@gmail.com>",
    description=DESCRIPTION,
    long_description=LONG_DESCRIPTION,
    packages=find_packages(),
    rust_extensions=[RustExtension("new_york_calculate.new_york_calculate", debug=False)],
    install_requires=["numpy"],
    keywords=["python", 'first package'],
    classifiers=[]
)
