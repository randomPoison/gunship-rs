<a name="v0.0.3"></a>
### v0.0.3 (2015-12-07)


#### Breaking Changes

* **parse-collada:**  rename PrimitiveType to PrimitiveElements ([f1cd6b4a](https://github.com/excaliburHisSheath/gunship-rs/commit/f1cd6b4af8b78ce2055da7e7e81f793fdc8c1306), breaks [#](https://github.com/excaliburHisSheath/gunship-rs/issues/))
* **polygon::MeshBuilder:** remove `Mesh::from_raw_data()` and replace it's functionality with `MeshBuilder`. ([907bf23c](https://github.com/excaliburHisSheath/gunship-rs/commit/907bf23cb8fca9c87c8d8aeff69a2646212ac8be))

#### Features

* **parse-collada:**
  *  add implementation for primitive elements ([953ac4bd](https://github.com/excaliburHisSheath/gunship-rs/commit/953ac4bdc5583fbf0551e799d1f6f947dc792ecf))
  *  better support parsing normal data ([6edf9162](https://github.com/excaliburHisSheath/gunship-rs/commit/6edf916283452a8d0f39a692e4810eb03dd66df2))
* **polygon-math::Vector2:**
  *  add Vector2::as_ref() ([d7eda56c](https://github.com/excaliburHisSheath/gunship-rs/commit/d7eda56c15fa560e081f2138d8c3b576e26e43ff))
  *  add Vector2 type and other utilities ([137f3c36](https://github.com/excaliburHisSheath/gunship-rs/commit/137f3c36299edb5b62ab8bd053e09d786a9c22ea))
* **polygon::MeshBuilder:**
  *  support texcoord vertex attributes ([03e875c7](https://github.com/excaliburHisSheath/gunship-rs/commit/03e875c74425becfe4bef95759d33a0a35e2fda0))
  *  add mesh builder system ([907bf23c](https://github.com/excaliburHisSheath/gunship-rs/commit/907bf23cb8fca9c87c8d8aeff69a2646212ac8be))
* **resource::collada:**
  *  process texcoord vertex data ([a87e2c96](https://github.com/excaliburHisSheath/gunship-rs/commit/a87e2c962d2de2f23162e061295603f36e25a365))
  *  more robust collada mesh processing ([bb075efa](https://github.com/excaliburHisSheath/gunship-rs/commit/bb075efa155132d8a864fa0f1042a8714a744dcd))

#### Bug Fixes

* **parse-collada:**  rename PrimitiveType to PrimitiveElements ([f1cd6b4a](https://github.com/excaliburHisSheath/gunship-rs/commit/f1cd6b4af8b78ce2055da7e7e81f793fdc8c1306), breaks [#](https://github.com/excaliburHisSheath/gunship-rs/issues/))
