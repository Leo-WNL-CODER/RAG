from fastapi import FastAPI, UploadFile
from unstructured.partition.auto import partition
from io import BytesIO
app = FastAPI()

@app.post("/parse")
async def parse_file(file:UploadFile):
    contents = await file.read()
    # Parse directly from memory (no temp file)
    elements = partition(
        file=BytesIO(contents),
        file_filename=file.filename
    )

    text = ""

    for el in elements:
        if hasattr(el, "text") and el.text:
            text += el.text + "\n"

    return {"text": text}
