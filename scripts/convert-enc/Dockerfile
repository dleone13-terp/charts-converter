FROM ghcr.io/dirkwa/signalk-charts-provider-simple/charts-toolbox:1.0.0

# Python 3 is already present in this image; no additional packages needed.
COPY convert.py /usr/local/bin/convert.py

WORKDIR /data
ENTRYPOINT ["python3", "/usr/local/bin/convert.py"]
