## script names:

### reference script:

`r_ScriptName.gd`

### value script:

`v_StrictName.gd`

### behaviour script: 

`ScriptName.gd`

## script content:

- no `$NodeName`
- all values should be statically typed as much as possible
- can use type inference (`:=`)
- `class_name`s should match script names

### reference script:

- no `class_name`
- extends `Node`
- only contains `@export var var_name: TypeName` statements

### value script

- has `class_name`
- extends `Node`
- only contains `@export var var_name: TypeName` statements
  
### behaviour scipt

- has a `class_name`
- extends `Node`

## where to place scripts

### reference script

- on a godot node, like `CharacterBody2D`, `Camera`, `AnimatedSprite2D`, etc
- this node may have children

### value script

- on a `Node`
- this node may not have children
- this node can be called a "value node"
- this node should be saved as a scene

### behaviour script

- on a `Node`
- this node may not have children
- this node can be called a "behaviour node" 
- this node should be saved as a scene

## nodes

- nodes can be grouped together by being children of a `Node`, `Node2D` or `Node3D`. `Node2D`s must be used for `y_sort_enabled`.
- these "grouping nodes" should not have scripts on them

## scenes

- if one would normally want to inherit (by creating a scene variant) from one of these scenes
- the new scene should be composed of the base scene and add other value no
- most instances of scenes will need to have `editable_children` on, this is fine

<u>editable children should not only be changed in the following ways:</u>

- changing the values in `@export var`s
- adding reference scripts

- if one would want to delete nodes from a scene instance, that scene instance should not be used
- a new base could be crated but this would be dangerously close to the failings of inheritance
- instead, composition should be used, composing using the resuable behaviour nodes and other components