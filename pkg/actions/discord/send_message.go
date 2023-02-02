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

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/expr"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
)

func SendMessageLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (actions.Action, error) {
	c := SendMessageConfig{
		Resource: "discord",
	}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	var resources resource.Resources
	if err := resolve.Resolve(resolver,
		"resource:lookup", &resources); err != nil {
		return nil, err
	}

	s, err := resource.Get[*discordgo.Session](resources, c.Resource)
	if err != nil {
		return nil, err
	}

	return SendMessageAction(s, &c), nil
}

func SendMessageAction(
	s *discordgo.Session,
	config *SendMessageConfig) actions.Action {
	return func(ctx context.Context, data actions.Data) (interface{}, error) {
		channelID, err := expr.EvalAsStringE(config.ChannelID, data)
		if err != nil {
			return nil, fmt.Errorf("expression %q did not evaluate a string: %w", config.ChannelID.Expr(), err)
		}

		content, err := expr.EvalAsStringE(config.Content, data)
		if err != nil {
			return nil, fmt.Errorf("expression %q did not evaluate a string: %w", config.Content.Expr(), err)
		}

		msg, err := s.ChannelMessageSend(channelID, content)
		if err != nil {
			return nil, fmt.Errorf("error sending Discord message: %w", err)
		}

		return msg.ID, nil
	}
}
