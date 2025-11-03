# Use a slim Python base image
FROM python:3.12-slim


# Set the working directory inside the container
WORKDIR /app

# Copy your application files into the container
COPY . /app


RUN pip install -r requirements.txt 

# Expose the port your FastAPI application will run on
EXPOSE 80

# Use uv run to execute uvicorn within the virtual environment
CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "80"]