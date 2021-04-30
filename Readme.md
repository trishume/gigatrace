# Proof-of-concept for a gigabyte-scale trace viewer

This repo includes:
- A memory-efficient representation for event traces
- An unusually simple and memory-efficient range aggregation index data structure (`IForestIndex`) for zooming traces of billions of events at 60fps
- A proof-of-concept Druid UI to demo efficient trace zooming, that isn't remotely useable as a real trace viewer.

It's a tech demo for the data structure described in [this blog post](https://thume.ca/2021/03/14/iforests/)
