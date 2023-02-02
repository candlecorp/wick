/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package discord

import (
	"context"
	"fmt"

	"github.com/bwmarrin/discordgo"

	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
)

type SessionConfig struct {
	Token string `mapstructure:"token" validate:"required"`
	Open  bool   `mapstructure:"open"`
}

// Session is the NamedLoader for a Discord bot session.
func Session() (string, resource.Loader) {
	return "nanobus.resource.discord/v1", SessionLoader
}

func SessionLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (interface{}, error) {
	var c SessionConfig
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	dg, err := discordgo.New("Bot " + c.Token)
	if err != nil {
		return nil, err
	}

	if c.Open {
		// Open a websocket connection to Discord and begin listening.
		err = dg.Open()
		if err != nil && err != discordgo.ErrWSAlreadyOpen {
			dg.Close()
			return nil, fmt.Errorf("error opening connection: %w", err)
		}
	}

	return dg, nil
}
