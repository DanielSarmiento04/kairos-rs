from . import app

@app.get("/")
async def read_root():
    return {"Hello": "World"}