# Buffered palletizer simulator

This webapp simulates stacking a feed of magazine bundles of varying height onto pallets.
The bundles need to be stacked uniformly so that the pallet is stable enough to be transported.
It was  used to analyze options and drive purchase decisions at a large printing company.

This application takes an excel file of a sequence of bundle sizes, simulates the stack sizes onto the pallets and displays them in an easy to analyze diagram.
Stacking of bundles that are too small is handled by a simulated presorter with buffers (default 4) .

## Tech stack
- SvelteKit for frontend
- Rust for the simulation environment
- wasm_bindgen to interface between sveltekit and compiled rust webassembly
- AI assistance was used for the visual aspect of the frontend

The entire webapp can be compiled to static html files. All simulation and data remains fully on-device.
