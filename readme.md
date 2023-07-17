# Img

#### Collectively share photo and video albums at their original quality, completely sovereign

## How To Use It
#### Make a new Collection
You can start a new collection of photos/videos either from the homepage
![image](https://github.com/jaybutera/img/assets/2101658/55f04e3e-b444-4924-9d93-7f48fc463544)

Or from any other page with "New Collection" in the top left corner. Just upload some initial pictures/videos to start it.



## What is it?
Img is essentially an open wiki for media. Anybody can create a topic by adding photos to it. The topic is an easily accessible url by anyone that knows it exists. Multiple people can upload to a single topic. It's an easy way to share photos with friends around an event, like a trip you took together, or an idea like cool art you find.

Img also makes downloading the images easy so they aren't locked in the cloud. Raw images are served at an endpoint which other tools can access and build on top of. Such tools may provide better UI's for viewing and organizing photos, or display collections in a native app. Sangha stores images without compressing, the original quality is preserved.

## Why does this exist?
Currently there's no organized and secure way to share photos. The choices are to host on a cloud like Google Photos, but the photos are then compressed and difficult to get back off the cloud and it's also not easy to share an album and have people add to it. Group messaging apps like Telegram and Signal are also used, but photos are not organized and get buried under new messages. Photos from these apps are also only stored on people's phones and so may be lost. And again it's not easy to share with new people.

## How?
Upon hosting your own sovereign server at example.com, to start a collection around your trip to alaska with friends, just visit `example.com/alaska/new` and upload your photos. Visit `example.com/alaska` to see them.

## Future Features
Eventually it would be nice to have a @user tag option in the url. Collections under a user tag can be permissioned as configured by the user. A user can decide who can see a collection and who can add to it. However sangha is designed to be small and do one thing well. User tags would require coupling the tool to a specific identity platform like Twitter or ActivityPub or ENS.
