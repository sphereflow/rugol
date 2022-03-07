# HELP
---
## CellType
There are 9 CellTypes: NoCell and CellType A to H.
Each CellType has its own color, value of type: FieldType and convolution kernel,
which are all edittable through the ui.
There is a CellType matrix which is used to display the game state.
## FieldType
There is a matrix which contains FieldType values.
These are componentwise multiplied and accumulated.
Currently there are only 2 FieldType's implemented:
- u8
- i8
## Rules
A rule consists of: an initial CellType, a next CellType and a range of type FieldType.
If a cell's CellType matches a rule and the accumulated value lies in the rules range,
then there will occur a transition from: initial CellType -> new CellType defined by the rule
the new CellType's value will then be written into the FieldType matrix.
Rules are stored linearly in a Vec.
Rules are applied top to bottom.
Only the first rule that matches is applied.
