AUREX-16++ AI HANDOFF DOCUMENT

This document defines the canonical state of Aurex-16++.

If context is lost, this file restores architectural truth.

Project Overview

Aurex-16++ is a deterministic 2D fantasy console platform built in Rust.

It is designed to:

Be hardware-inspired

Be LLM-friendly

Enforce strict constraints

Support trophies

Support guided game creation

Remain 2D-only

Core System Summary
Frame Model

60 FPS fixed

200k ops per frame cap

Deterministic execution

Memory

512 KB WRAM

1 MB VRAM (partitioned)

256 KB ASU sample RAM

Graphics

Tile + sprite based

Mode 7 support

Line effects capped

256 colors max on screen

Audio

16 voices

PCM + synth hybrid

Built-in sequencer

Echo + limiter

DMA

Hard capped

Reject with visible warning

4 commands per frame max

Entities

256 ECSU slots

Standardized structure

Diagnostics

PDU tracks everything performance-related

Achievements

Built-in system

Trophy metadata in cart

Unlock API

GCU

Built-in guided creation system

Visible in Library

Architectural Rule

If new features contradict:

Determinism

2D-only philosophy

Hardware-style constraints

They are rejected.

Immediate Rebuild Starting Point

Begin with:

Clean Rust project

Frame loop

PDU

Ops enforcement

WRAM scaffold

Everything else builds on top.