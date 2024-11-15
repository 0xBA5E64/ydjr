#/bin/bash
mkdir test-files
cd test-files
yt-dlp --merge-output-format mkv --embed-info-json -o "1.%(ext)s" https://www.youtube.com/watch?v=FBgLytbB-uE
yt-dlp --merge-output-format mkv --embed-info-json -o "2.%(ext)s" https://www.youtube.com/watch?v=l1uaX5hBByw
yt-dlp --merge-output-format mkv --embed-info-json -o "3.%(ext)s" https://www.youtube.com/watch?v=KSTQCf6BAFM
yt-dlp --merge-output-format mkv --embed-info-json -o "4.%(ext)s" https://www.youtube.com/watch?v=wz2opOBqClo
touch not_a_video.file