# ThorUI

ThorUI describes dual-screen games and applications as one experience spread across independently driven surfaces. This language keeps hardware, browser, and product concepts distinct.

## Experience

**Experience**:
A game or application built with ThorUI and presented as one product to the user.
_Avoid_: App, title, client

**Session**:
One active run of an experience, including the state shared by all of its surfaces.
_Avoid_: Instance, process, game session

**Projection**:
The part of a session presented on one surface. Two projections may show different views of the same session.
_Avoid_: Page, scene, screen content

## Device

**Surface**:
An independently sized, refreshed, and interactive visual area available to a session.
_Avoid_: Window, canvas, monitor

**Main Surface**:
The surface used for the primary task or game world. It is normally the Thor's upper display.
_Avoid_: Top screen, primary window

**Companion Surface**:
The surface used for supporting controls, detail, status, or secondary tasks. It is normally the Thor's lower display.
_Avoid_: Bottom screen, secondary window

**Surface Profile**:
Observed properties of one surface, including size, pixel density, refresh behavior, and available interaction methods.
_Avoid_: Screen config, display constants

**Device Profile**:
The set of observed capabilities and surface profiles for one device and host combination.
_Avoid_: User agent, device constants

## Interaction

**Input Sample**:
A timestamped observation from a physical input source before meaning is assigned to it.
_Avoid_: Event, button press

**Action**:
A semantic user intent produced from one or more input samples, such as Confirm, Navigate, or Pause.
_Avoid_: Key, binding, command

**Interaction Mode**:
The input family currently guiding feedback and focus behavior: controller, touch, or keyboard and pointer.
_Avoid_: Input type, control mode

## Design System

**Design Token**:
A named semantic design decision shared by surfaces and presentation methods.
_Avoid_: Magic value, style constant

**Control Primitive**:
ThorUI-owned interaction behavior for a standard control, independent of one visual identity.
_Avoid_: Widget, copied component

**UI Recipe**:
Workspace-owned Rust and style source that gives control primitives a specific structure and visual identity.
_Avoid_: Theme component, framework widget

**Registry Item**:
A versioned source package that installs a UI recipe and its declared dependencies into a workspace.
_Avoid_: UI dependency, black-box component

## Runtime

**Authority**:
The single owner of canonical session state at a given time.
_Avoid_: Master, server, main window

**Surface Peer**:
A runtime participant that gathers input for and presents one surface.
_Avoid_: Client, view process, renderer

**Capability Report**:
A saved record of observed host and device behavior produced by the hardware probe.
_Avoid_: Spec sheet, compatibility guess
