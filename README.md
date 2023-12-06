# Pyrite

A "game engine" focused on heavily on modularity and extensibility. <br/>

The goal of this project is to provide all the pieces you would want for a game engine that are designed to be put together by the game.
With this architecture, it makes larger projects more explicit about importing each small module and more cognizant of what is running their high level code.

### Main Focus

- **Modularity** - often times engines have very opinionated workflows, we avoid that by providing a highly modular
  system.
- **Ease of use** - we want to be highly customizable while still offering to libraries to develop quickly.
- **Explicit** - every desired resource, system, and parallel run conditions need to be defined by the game, there are no plugins.
- **Parallelism** - using a customizable dependency graph, modules can define specific dependencies for subtasks to run as parallel as possible.

### Current State

Pyrite is in a very early stage of active development and is being built mainly for a future game. The main focus of the
engine though is to allow for future games to quickly be made from it. With these points in mind, performance and
developer experience are the main factors for consideration currently. <br/>

Everything is currently running synchronously without the whole async system dependency graph and scheduler. However, as I encounter API choices for modules that
require asynchronous consideration. I take note of all of these so that when I refactor the main app structure to allow for the extra api flexebility when it
comes to parallel scheduling, the API will allow for high customizability and should all feel good as a develop. Naturally debugging asynchronous code that you are
relying on a scheduler to run can introduce bugs which can be hard to debug. This is why a future goal when it comes to making parallelism better, is to enable
a mode that would allow your program to host a local debug webapp that would show how a frame was executed by the scheduler and what the resolved dependency graph looks like.

### Thanks

This project was initially inspired mainly by [Bevy](https://github.com/bevyengine/bevy) for showing what rust is capable of
providing in the context of game development. Bevy also inspired some neat patterns that rust allows for and the general
project structure. Check out [taskflow](https://github.com/taskflow/taskflow) for how a well designed parallel task programing API can look like.
