from setuptools import Extension, setup

setup(ext_modules=[
    Extension(
        name="qtile_jack.jax",
        sources=["jax.c"],
        extra_compile_args=["-Wall", "-Wextra", "-Werror", "-Wno-unused"],
        extra_link_args=["-ljack", "-ljackserver"],
    ),
])
