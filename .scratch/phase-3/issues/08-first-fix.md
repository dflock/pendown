# The first fix

Status: ready-for-agent
Type: task
Blocked by: 04

## What

Whatever the numbers show. Expected, from the first look: every frame re-renders the whole window. Make the editor and console cached views, so a frame only rebuilds the toolbar and repaints the canvas. Also check the canvas uploads tiles only when they change.

## Done when

A slow run costs little more than an idle window, by the summary and the overlay, and Instant is as fast as before by `perf` and the scripted session.
