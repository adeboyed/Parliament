rm Parliament/*
rsync -av --exclude=".*" ../../Parliament/* Parliament
docker build -t adeboyed/parliament-sort .
rm -rf Parliament/*
