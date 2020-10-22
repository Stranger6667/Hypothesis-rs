from setuptools import setup
from setuptools_rust import Binding, RustExtension

setup(
    name="database",
    version="0.1",
    rust_extensions=[RustExtension("database", binding=Binding.PyO3)],
    zip_safe=False,
)
