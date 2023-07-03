package main

import (
	"fmt"
	"time"

	"github.com/kindlyfire/go-keylogger"
)

const (
	delayKeyfetchMS = 50
)

// the way toadlkjflksdjflksfjslkfjslfasdjkflskjfaslfkjaslfjsdlfjdflsflfslfjsdflsdflsflasfslfslfsalfd
// the way we should act toghater is to build some of the world best suitable  sofware ksdf
func main() {
	kl := keylogger.NewKeylogger()
	emptyCount := 0

	for {
		key := kl.GetKey()

		if !key.Empty {
			fmt.Printf("'%c' %d                     \n", key.Rune, key.Keycode)
		}

		emptyCount++

		fmt.Printf("Empty count: %d\r", emptyCount)

		time.Sleep(delayKeyfetchMS * time.Millisecond)
	}
}