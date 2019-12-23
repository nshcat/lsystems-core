### lsystems-core 
#### A rust implementation of parametric, stochastic and context-sensitive L-systems

#### Introduction

This library implements so-called [L-systems](https://en.wikipedia.org/wiki/L-system), which are parallel rewriting systems that can be used to model many things occuring in nature, such as plant life. By assigning geometric meaning to a chosen set of control characters and interpreting them using the concept of [turtle graphics](https://en.wikipedia.org/wiki/Turtle_graphics), pictures can be drawn based on evaluated systems, a feature which is often used to simulate plants.

This library is not immediately useful as a standalone tool, since it is not able to actually visualize the primitves derived from the evaluation of an L-system. It's purpose is to merely serve as the basis for applications that wish to visualize L-systems by providing the means necessary to construct and evaluate them.

#### Basic Usage

##### Evaluating L-systems

Creating and evaluation an L-system is simple:

```rust
let mut lsystem = LSystem::new();        // Create empty system

let params = DrawingParameters { .. } ;  // Setup settings (see below)
lsystem.set_drawing_parameters(params);
lsystem.set_iteration_depth(3); 

lsystem.parse(                           // Set axiom and rewriting
     "A",                                // rules
     vec![ "A -> FAB", "B -> +F" ]
);

lsystem.iterate();                       // Evaluate and draw L-system
lsystem.interpret();

lsystem.drawing_result. /*..*/           // Access generated primitives
```

##### Drawing Parameters
The drawing parameters are a set of values that control how the drawing operations assigned to the generated control characters are acted upon. The following settings are supported:

| Setting  | Description |
| ------------- | ------------- |
| Starting Position  | Controls the starting position of the turtle |
| Starting angle | Sets the starting angle of the turtle |
| Angle delta | Angle increment/decrement used for turning operations, such as "turn right" |
| Step | How long a single drawn line is supposed to be |
| Line width | Initial thickness of drawn lines, in pixels |
| Line width delta | Increment/decrement used for operations that modify the line thickness |

##### Axiom and Rules

The axiom and rules form the formal grammar of the L-system and are therefore the most important part of any L-system. Since L-systems are rewriting system, the basic concept of rules is the following:
```
/*thing to replace*/ -> /*stuff to replace thing with*/
```
The iteration engine inside this library will take the so called *axiom*, an initial string of characters, and try to apply each of the registered rules until the specified iteration depth is reached.

This library supports many different types of rules, from basic deterministic ones to stochastic and parametric ones.

###### Deterministic Rules
Deterministic rules are the simplest kind of rules. They match a single character on the left-hand side, and replace them with a simple string of characters:
```
A -> AB     /* Replaces each A with the string AB */
B -> C      /* Replaces each B with a single C */
```
As the name implies, the application of this type of rule is always deterministic: Should multiple rules match the same character in the string, the iteration engine will always choose the first rule; no non-deterministic choice is done.

###### Context-Sensitive Rules
Sometimes, a model requires the application of certain rules on a character to be based on its immediate neighbours (the so-called *context*). This can be achieved by using context-sensitive rules, which allow the left-hand side to specify which characters have to be present on the left and right of a symbol in order for it to be eligible to be replaced:
```
A < B > C  -> D       /* B must be surrounded by A and C */
A < B      -> E       /* B must have an A to its left */
    B > C  -> F       /* B must have an C to its right */
```
It's important to notice here that those types of context patterns may only specify at most one character per side.

###### Parametric Rules
For more complex models, this library also implements parametric rules, which allow the usage of variables and expressions in rules, as well as rule matching based on arbitrary  boolean conditions. 
A central concept is that any symbol in the L-system string can have any number of parameters:
```
A(3.3, 1.0, 1e-33)
```
Those parameters belong to the symbol `A`, and rules can match on them by binding names to them and stating boolean expressions that act as conditions:
```
A(x) : x > 3 -> FF   /* Only matches A(x) if x is greater than 3 */
```
Additionally, rules can perform arbitrary arithmetic using the parameters:
```
A(x) -> A(x+1)       /* Increment the parameter by 1 */
```
Of course, these two features can be combined, which allows complex models:
```
A(x, y) : x > y  -> A(x-1, y)    /* Decrement x until x == y */
A(x, y) : x <= y -> A(x, y)
```
The drawing operations for the turtle graphics can all be parameterized by exactly one parameter, which influences their operation. For example, if `F` denotes the symbol for "go forward and draw a line", `F(0.2)` would draw a line with a length of 0.2 units.

###### Stochastic Rules
While the types of rules explained above are deterministic in nature, sometimes a model requires some degree of randomness. For this reason, this library also implements stochastic rules, which allow the matching of rules to depend on chance.
```
A(x) : 0.5 -> A(x-1)  /* Randomly increment and decrement parameter */
A(x) : 0.5 -> A(x+1)
```
It should be noted that the probability values do not work as absolute values - rather they are arbitrary numerical weights used to select a rule using a russian-roulette type algorithm. This means that if a rule with a probability `p` is the only rule that even matches, it will always be chosen. If a model requires the possibility of no rule to be chosen, identity rules can be added that map a symbol and its parameters onto itself. Additionally, deterministic rules always take precedent over any stochastic rules, if they match.

Of course, stochastic rules can be combined with the boolean conditions provided by the parametric rules support:
```
A(x) : x > 0 : 0.5 -> A(x-1) 
A(x) : x > 0 : 0.5 -> A(x+1)
```


##### Turtle Graphics Operations
The turtle graphics component of this library, which is responsible for drawing the L-system, supports a wide array of commands and operations, which can be assigned to any character in the alphabet of the L-system:

| Operation | Description |
| ------------- | ------------- |
| Ignore | Do nothing |
| Forward | The turtle moves forward while drawing a line |
| Forward  (no draw)| The turtle moves forward without drawing a line |
| Forward (contracting) | Draws a line of a length dependant on the iteration depth (l_i = l^i). This can be used to keep the size of an L-system constant with different iteration depths|
|Turn Right, Left, Around| Turns the turtle by the angle delta either left or right, or turns around 180° |
|Pitch Down, Up | Pitches the turtle nose down or up |
| Roll Left, Right | Turns the turtle around its direction vector |
|Save, Load State| Pushes and Pops turtle state (position, direction, ...) on/from a stack. This allows the turtle to remember state and return to it, e.g. after drawing a part of the L-system|
 |Begin, End Polygon, Submit Vertex| Used to draw simple polygons using triangle fans|
 |Increment, Decrement Color | Modify the index into the color palette |
 |Increment, Decrement Line Width| Modify the line width|
 
 ##### Supported Primitives
 
 ###### Lines
 ###### Simple Polygons
 ###### Bicubic Bezier Patches
 