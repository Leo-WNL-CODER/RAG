import io
import magic
from fastapi import FastAPI, UploadFile, HTTPException
from pypdf import PdfReader
from docx import Document

app = FastAPI()

@app.post("/parse")
async def parse_file(file: UploadFile):
    contents = await file.read()
    
    # Detect file type
    mime = magic.from_buffer(contents[:2048], mime=True)
    
    text = ""

    if mime == "application/pdf":
        try:
            reader = PdfReader(io.BytesIO(contents))
            for page in reader.pages:
                page_text = page.extract_text()
                if page_text:
                    text += page_text + "\n"
        except Exception as e:
            raise HTTPException(status_code=500, detail=f"PDF parse error: {str(e)}")

    elif mime in ("application/vnd.openxmlformats-officedocument.wordprocessingml.document",):
        try:
            doc = Document(io.BytesIO(contents))
            for para in doc.paragraphs:
                if para.text.strip():
                    text += para.text + "\n"
        except Exception as e:
            raise HTTPException(status_code=500, detail=f"DOCX parse error: {str(e)}")

    elif mime.startswith("text/"):
        text = contents.decode("utf-8", errors="ignore")

    else:
        raise HTTPException(status_code=415, detail=f"Unsupported file type: {mime}")

    return {"text": text}
