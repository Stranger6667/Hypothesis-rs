from setuptools import setup
from setuptools_rust import Binding, RustExtension

setup(
    name="charmap",
    version="0.1",
    rust_extensions=[RustExtension("charmap", binding=Binding.PyO3)],
    zip_safe=False,
)
