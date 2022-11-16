/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package gorm

import (
	"fmt"
	"reflect"
	"strconv"
	"strings"

	"gorm.io/gorm/schema"
	"gorm.io/gorm/utils"

	"github.com/nanobus/nanobus/pkg/spec"
)

type (
	Namespaces map[string]*Namespace

	Namespace map[string]*schema.Schema

	Processor struct {
		types map[string]*spec.Type
		pairs map[string]Pair
		namer schema.Namer
	}

	Pair struct {
		T *spec.Type
		S *schema.Schema
	}
)

func SpecToSchemas(s spec.Namespaces) Namespaces {
	ns := make(Namespaces, len(s))

	return ns
}

var mapType = reflect.TypeOf(map[string]interface{}{})

func (p *Processor) ConvertTypes(types []*spec.Type) error {
	for _, t := range types {
		p.types[t.Name] = t
	}
	for _, t := range types {
		if t.Name == "Customer" || t.Name == "Address" {
			if _, err := p.TypeToSchema(t); err != nil {
				return err
			}
		}
	}

	return nil
}

func NewProcessor(namer schema.Namer) Processor {
	return Processor{
		types: make(map[string]*spec.Type),
		pairs: make(map[string]Pair),
		namer: namer,
	}
}

// TypeToSchema get data type from dialector with extra schema table
func (p *Processor) TypeToSchema(t *spec.Type) (Pair, error) {
	pair, ok := p.pairs[t.Name]
	if ok {
		return pair, nil
	}

	tableName := annotationValue(t, "entity", "table", "")
	if tableName == "" {
		tableName = p.namer.TableName(t.Name)
	}

	s := &schema.Schema{
		Name:           t.Name,
		ModelType:      mapType,
		Table:          tableName,
		FieldsByName:   map[string]*schema.Field{},
		FieldsByDBName: map[string]*schema.Field{},
		Relationships:  schema.Relationships{Relations: map[string]*schema.Relationship{}},
	}
	pair = Pair{
		T: t,
		S: s,
	}
	p.pairs[t.Name] = pair

	for _, f := range t.Fields {
		parsed, err := parseField(s, f)
		if err != nil {
			return pair, err
		}
		s.Fields = append(s.Fields, parsed)
	}

	// for i := 0; i < modelType.NumField(); i++ {
	// 	if fieldStruct := modelType.Field(i); ast.IsExported(fieldStruct.Name) {
	// 		if field := schema.ParseField(fieldStruct); field.EmbeddedSchema != nil {
	// 			schema.Fields = append(schema.Fields, field.EmbeddedSchema.Fields...)
	// 		} else {
	// 			schema.Fields = append(schema.Fields, field)
	// 		}
	// 	}
	// }

	for _, field := range s.Fields {
		if field.DBName == "" && field.DataType != "" {
			field.DBName = p.namer.ColumnName(s.Table, field.Name)
		}

		if field.DBName != "" {
			// nonexistence or shortest path or first appear prioritized if has permission
			if v, ok := s.FieldsByDBName[field.DBName]; !ok || ((field.Creatable || field.Updatable || field.Readable) && len(field.BindNames) < len(v.BindNames)) {
				if _, ok := s.FieldsByDBName[field.DBName]; !ok {
					s.DBNames = append(s.DBNames, field.DBName)
				}
				s.FieldsByDBName[field.DBName] = field
				s.FieldsByName[field.Name] = field

				if v != nil && v.PrimaryKey {
					for idx, f := range s.PrimaryFields {
						if f == v {
							s.PrimaryFields = append(s.PrimaryFields[0:idx], s.PrimaryFields[idx+1:]...)
						}
					}
				}

				if field.PrimaryKey {
					s.PrimaryFields = append(s.PrimaryFields, field)
				}
			}
		}

		if of, ok := s.FieldsByName[field.Name]; !ok || of.TagSettings["-"] == "-" {
			s.FieldsByName[field.Name] = field
		}

		//field.setupValuerAndSetter()
	}

	prioritizedPrimaryField := s.LookUpField("id")
	if prioritizedPrimaryField == nil {
		prioritizedPrimaryField = s.LookUpField("ID")
	}

	if prioritizedPrimaryField != nil {
		if prioritizedPrimaryField.PrimaryKey {
			s.PrioritizedPrimaryField = prioritizedPrimaryField
		} else if len(s.PrimaryFields) == 0 {
			prioritizedPrimaryField.PrimaryKey = true
			s.PrioritizedPrimaryField = prioritizedPrimaryField
			s.PrimaryFields = append(s.PrimaryFields, prioritizedPrimaryField)
		}
	}

	if s.PrioritizedPrimaryField == nil && len(s.PrimaryFields) == 1 {
		s.PrioritizedPrimaryField = s.PrimaryFields[0]
	}

	for _, field := range s.PrimaryFields {
		s.PrimaryFieldDBNames = append(s.PrimaryFieldDBNames, field.DBName)
	}

	for _, field := range s.Fields {
		if field.HasDefaultValue && field.DefaultValueInterface == nil {
			s.FieldsWithDefaultDBValue = append(s.FieldsWithDefaultDBValue, field)
		}
	}

	if field := s.PrioritizedPrimaryField; field != nil {
		switch field.GORMDataType {
		case schema.Int, schema.Uint:
			if _, ok := field.TagSettings["AUTOINCREMENT"]; !ok {
				if !field.HasDefaultValue || field.DefaultValueInterface != nil {
					s.FieldsWithDefaultDBValue = append(s.FieldsWithDefaultDBValue, field)
				}

				field.HasDefaultValue = true
				field.AutoIncrement = true
			}
		}
	}

	// callbacks := []string{"BeforeCreate", "AfterCreate", "BeforeUpdate", "AfterUpdate", "BeforeSave", "AfterSave", "BeforeDelete", "AfterDelete", "AfterFind"}
	// for _, name := range callbacks {
	// 	if methodValue := modelValue.MethodByName(name); methodValue.IsValid() {
	// 		switch methodValue.Type().String() {
	// 		case "func(*gorm.DB) error": // TODO hack
	// 			reflect.Indirect(reflect.ValueOf(s)).FieldByName(name).SetBool(true)
	// 		default:
	// 			logger.Default.Warn(context.Background(), "Model %v don't match %vInterface, should be `%v(*gorm.DB) error`. Please see https://gorm.io/docs/hooks.html", schema, name, name)
	// 		}
	// 	}
	// }

	// Cache the schema
	// if v, loaded := cacheStore.LoadOrStore(schemaCacheKey, schema); loaded {
	// 	s := v.(*Schema)
	// 	// Wait for the initialization of other goroutines to complete
	// 	<-s.initialized
	// 	return s, s.err
	// }

	// defer func() {
	// 	if schema.err != nil {
	// 		logger.Default.Error(context.Background(), schema.err.Error())
	// 		cacheStore.Delete(modelType)
	// 	}
	// }()

	// if _, embedded := schema.cacheStore.Load(embeddedCacheKey); !embedded {
	for _, field := range s.Fields {
		fieldType, _ := t.Field(field.Name)
		if field.DataType == "" && (field.Creatable || field.Updatable || field.Readable) {
			if _, err := p.parseRelation(fieldType, s, field); err != nil {
				return pair, err
			} else {
				s.FieldsByName[field.Name] = field
			}
		}

		// fieldValue := reflect.New(field.IndirectFieldType)
		// fieldInterface := fieldValue.Interface()
		// if fc, ok := fieldInterface.(CreateClausesInterface); ok {
		// 	field.Schema.CreateClauses = append(field.Schema.CreateClauses, fc.CreateClauses(field)...)
		// }

		// if fc, ok := fieldInterface.(QueryClausesInterface); ok {
		// 	field.Schema.QueryClauses = append(field.Schema.QueryClauses, fc.QueryClauses(field)...)
		// }

		// if fc, ok := fieldInterface.(UpdateClausesInterface); ok {
		// 	field.Schema.UpdateClauses = append(field.Schema.UpdateClauses, fc.UpdateClauses(field)...)
		// }

		// if fc, ok := fieldInterface.(DeleteClausesInterface); ok {
		// 	field.Schema.DeleteClauses = append(field.Schema.DeleteClauses, fc.DeleteClauses(field)...)
		// }
	}
	// }

	return pair, nil
}

func (p *Processor) parseRelation(specField *spec.Field, s *schema.Schema, field *schema.Field) (*schema.Relationship, error) {
	var (
		err error
		//fieldValue = reflect.New(field.IndirectFieldType).Interface()
		relation = &schema.Relationship{
			Name:   field.Name,
			Field:  field,
			Schema: s,
			// foreignKeys: toColumns(field.TagSettings["FOREIGNKEY"]),
			// primaryKeys: toColumns(field.TagSettings["REFERENCES"]),
		}
	)

	// cacheStore := schema.cacheStore

	//specField, _ := t.Field(field.Name)
	fmt.Println(specField.Name)
	fieldType := specField.Type
	//fmt.Println(fieldType)
	if fieldType.Kind == spec.KindOptional {
		fieldType = fieldType.OptionalType
	}
	//fmt.Println(fieldType)
	pair, ok := p.pairs[fieldType.Type.Name]
	if !ok {
		fmt.Println(fieldType.Type.Name)
		pair, err = p.TypeToSchema(p.types[fieldType.Type.Name])
		if err != nil {
			return nil, err
		}
	}
	fieldSchema := pair.S

	relation.FieldSchema = fieldSchema

	relation.Type = schema.HasOne

	// if polymorphic := field.TagSettings["POLYMORPHIC"]; polymorphic != "" {
	// 	err = p.buildPolymorphicRelation(s, relation, field, polymorphic)
	// } else if many2many := field.TagSettings["MANY2MANY"]; many2many != "" {
	// 	err = p.buildMany2ManyRelation(s, relation, field, many2many)
	// } else if belongsTo := field.TagSettings["BELONGSTO"]; belongsTo != "" {
	// 	err = p.guessRelation(s, relation, field, guessBelongs)
	// } else {
	// 	switch field.IndirectFieldType.Kind() {
	// 	case reflect.Struct:
	// 		err = p.guessRelation(s, relation, field, guessGuess)
	// 	case reflect.Slice:
	// 		err = p.guessRelation(s, relation, field, guessHas)
	// 	default:
	// 		return nil, fmt.Errorf("unsupported data type %v for %v on field %s", relation.FieldSchema, s, field.Name)
	// 	}
	// }

	// if relation.Type == schema.RelationshipType("has") {
	// 	// don't add relations to embedded schema, which might be shared
	// 	if relation.FieldSchema != relation.Schema && relation.Polymorphic == nil && field.OwnerSchema == nil {
	// 		relation.FieldSchema.Relationships.Relations["_"+relation.Schema.Name+"_"+relation.Name] = relation
	// 	}

	// 	switch field.IndirectFieldType.Kind() {
	// 	case reflect.Struct:
	// 		relation.Type = schema.HasOne
	// 	case reflect.Slice:
	// 		relation.Type = schema.HasMany
	// 	}
	// }

	s.Relationships.Relations[relation.Name] = relation
	switch relation.Type {
	case schema.HasOne:
		s.Relationships.HasOne = append(s.Relationships.HasOne, relation)
	case schema.HasMany:
		s.Relationships.HasMany = append(s.Relationships.HasMany, relation)
	case schema.BelongsTo:
		s.Relationships.BelongsTo = append(s.Relationships.BelongsTo, relation)
	case schema.Many2Many:
		s.Relationships.Many2Many = append(s.Relationships.Many2Many, relation)
	}

	return relation, nil
}

func parseField(s *schema.Schema, f *spec.Field) (*schema.Field, error) {
	var err error

	fieldType := reflectType(f.Type)

	var tagSettings string
	if a, ok := f.Annotation("gorm"); ok {
		tagSettings = a.Arguments[0].ValueString()
	}

	field := &schema.Field{
		Name:              f.Name,
		BindNames:         []string{f.Name},
		FieldType:         fieldType,
		IndirectFieldType: fieldType,
		//StructField:            mapType,
		Creatable:              true,
		Updatable:              true,
		Readable:               true,
		Tag:                    "", //fieldStruct.Tag,
		TagSettings:            schema.ParseTagSetting(tagSettings, ";"),
		Schema:                 s,
		AutoIncrementIncrement: 1,
		Comment:                f.Description,
	}

	// for field.IndirectFieldType.Kind() == reflect.Ptr {
	// 	field.IndirectFieldType = field.IndirectFieldType.Elem()
	// }

	//fieldValue := reflect.New(field.IndirectFieldType)
	// // if field is valuer, used its value or first fields as data type
	// valuer, isValuer := fieldValue.Interface().(driver.Valuer)
	// if isValuer {
	// 	if _, ok := fieldValue.Interface().(GormDataTypeInterface); !ok {
	// 		if v, err := valuer.Value(); reflect.ValueOf(v).IsValid() && err == nil {
	// 			fieldValue = reflect.ValueOf(v)
	// 		}

	// 		var getRealFieldValue func(reflect.Value)
	// 		getRealFieldValue = func(v reflect.Value) {
	// 			rv := reflect.Indirect(v)
	// 			if rv.Kind() == reflect.Struct && !rv.Type().ConvertibleTo(TimeReflectType) {
	// 				for i := 0; i < rv.Type().NumField(); i++ {
	// 					newFieldType := rv.Type().Field(i).Type
	// 					for newFieldType.Kind() == reflect.Ptr {
	// 						newFieldType = newFieldType.Elem()
	// 					}

	// 					fieldValue = reflect.New(newFieldType)

	// 					if rv.Type() != reflect.Indirect(fieldValue).Type() {
	// 						getRealFieldValue(fieldValue)
	// 					}

	// 					if fieldValue.IsValid() {
	// 						return
	// 					}

	// 					for key, value := range ParseTagSetting(field.IndirectFieldType.Field(i).Tag.Get("gorm"), ";") {
	// 						if _, ok := field.TagSettings[key]; !ok {
	// 							field.TagSettings[key] = value
	// 						}
	// 					}
	// 				}
	// 			}
	// 		}

	// 		getRealFieldValue(fieldValue)
	// 	}
	// }

	if dbName, ok := field.TagSettings["COLUMN"]; ok {
		field.DBName = dbName
	}

	if val, ok := field.TagSettings["PRIMARYKEY"]; ok && utils.CheckTruth(val) {
		field.PrimaryKey = true
	} else if val, ok := field.TagSettings["PRIMARY_KEY"]; ok && utils.CheckTruth(val) {
		field.PrimaryKey = true
	}

	if val, ok := field.TagSettings["AUTOINCREMENT"]; ok && utils.CheckTruth(val) {
		field.AutoIncrement = true
		field.HasDefaultValue = true
	}

	if num, ok := field.TagSettings["AUTOINCREMENTINCREMENT"]; ok {
		field.AutoIncrementIncrement, _ = strconv.ParseInt(num, 10, 64)
	}

	if v, ok := field.TagSettings["DEFAULT"]; ok {
		field.HasDefaultValue = true
		field.DefaultValue = v
	}

	if num, ok := field.TagSettings["SIZE"]; ok {
		if field.Size, err = strconv.Atoi(num); err != nil {
			field.Size = -1
		}
	}

	if p, ok := field.TagSettings["PRECISION"]; ok {
		field.Precision, _ = strconv.Atoi(p)
	}

	if s, ok := field.TagSettings["SCALE"]; ok {
		field.Scale, _ = strconv.Atoi(s)
	}

	if val, ok := field.TagSettings["NOT NULL"]; ok && utils.CheckTruth(val) {
		field.NotNull = true
	} else if val, ok := field.TagSettings["NOTNULL"]; ok && utils.CheckTruth(val) {
		field.NotNull = true
	}

	if val, ok := field.TagSettings["UNIQUE"]; ok && utils.CheckTruth(val) {
		field.Unique = true
	}

	if val, ok := field.TagSettings["COMMENT"]; ok {
		field.Comment = val
	}

	// default value is function or null or blank (primary keys)
	field.DefaultValue = strings.TrimSpace(field.DefaultValue)
	skipParseDefaultValue := strings.Contains(field.DefaultValue, "(") &&
		strings.Contains(field.DefaultValue, ")") || strings.ToLower(field.DefaultValue) == "null" || field.DefaultValue == ""
	t := f.Type
	if t.Kind == spec.KindOptional {
		t = f.Type.OptionalType
	}
	switch t.Kind {
	case spec.KindBool:
		field.DataType = schema.Bool
		if field.HasDefaultValue && !skipParseDefaultValue {
			if field.DefaultValueInterface, err = strconv.ParseBool(field.DefaultValue); err != nil {
				return nil, fmt.Errorf("failed to parse %s as default value for bool, got error: %v", field.DefaultValue, err)
			}
		}
	case spec.KindI8, spec.KindI16, spec.KindI32, spec.KindI64:
		field.DataType = schema.Int
		if field.HasDefaultValue && !skipParseDefaultValue {
			if field.DefaultValueInterface, err = strconv.ParseInt(field.DefaultValue, 0, 64); err != nil {
				return nil, fmt.Errorf("failed to parse %s as default value for int, got error: %v", field.DefaultValue, err)
			}
		}
	case spec.KindU8, spec.KindU16, spec.KindU32, spec.KindU64:
		field.DataType = schema.Uint
		if field.HasDefaultValue && !skipParseDefaultValue {
			if field.DefaultValueInterface, err = strconv.ParseUint(field.DefaultValue, 0, 64); err != nil {
				return nil, fmt.Errorf("failed to parse %s as default value for uint, got error: %v", field.DefaultValue, err)
			}
		}
	case spec.KindF32, spec.KindF64:
		field.DataType = schema.Float
		if field.HasDefaultValue && !skipParseDefaultValue {
			if field.DefaultValueInterface, err = strconv.ParseFloat(field.DefaultValue, 64); err != nil {
				return nil, fmt.Errorf("failed to parse %s as default value for float, got error: %v", field.DefaultValue, err)
			}
		}
	case spec.KindString:
		field.DataType = schema.String

		if field.HasDefaultValue && !skipParseDefaultValue {
			field.DefaultValue = strings.Trim(field.DefaultValue, "'")
			field.DefaultValue = strings.Trim(field.DefaultValue, `"`)
			field.DefaultValueInterface = field.DefaultValue
		}
	case spec.KindDateTime:
		field.DataType = schema.Time
	// case spec.KindType:
	// 	if _, ok := fieldValue.Interface().(*time.Time); ok {
	// 		field.DataType = schema.Time
	// 	} else if fieldValue.Type().ConvertibleTo(schema.TimeReflectType) {
	// 		field.DataType = schema.Time
	// 	} else if fieldValue.Type().ConvertibleTo(reflect.TypeOf(&time.Time{})) {
	// 		field.DataType = schema.Time
	// 	}
	case spec.KindBytes:
		field.DataType = schema.Bytes
	}

	field.GORMDataType = field.DataType

	// if dataTyper, ok := fieldValue.Interface().(schema.GormDataTypeInterface); ok {
	// 	field.DataType = schema.DataType(dataTyper.GormDataType())
	// }

	if v, ok := field.TagSettings["AUTOCREATETIME"]; ok || (field.Name == "CreatedAt" && (field.DataType == schema.Time || field.DataType == schema.Int || field.DataType == schema.Uint)) {
		if field.DataType == schema.Time {
			field.AutoCreateTime = schema.UnixTime
		} else if strings.ToUpper(v) == "NANO" {
			field.AutoCreateTime = schema.UnixNanosecond
		} else if strings.ToUpper(v) == "MILLI" {
			field.AutoCreateTime = schema.UnixMillisecond
		} else {
			field.AutoCreateTime = schema.UnixSecond
		}
	}

	if v, ok := field.TagSettings["AUTOUPDATETIME"]; ok || (field.Name == "UpdatedAt" && (field.DataType == schema.Time || field.DataType == schema.Int || field.DataType == schema.Uint)) {
		if field.DataType == schema.Time {
			field.AutoUpdateTime = schema.UnixTime
		} else if strings.ToUpper(v) == "NANO" {
			field.AutoUpdateTime = schema.UnixNanosecond
		} else if strings.ToUpper(v) == "MILLI" {
			field.AutoUpdateTime = schema.UnixMillisecond
		} else {
			field.AutoUpdateTime = schema.UnixSecond
		}
	}

	if val, ok := field.TagSettings["TYPE"]; ok {
		switch schema.DataType(strings.ToLower(val)) {
		case schema.Bool, schema.Int, schema.Uint, schema.Float, schema.String, schema.Time, schema.Bytes:
			field.DataType = schema.DataType(strings.ToLower(val))
		default:
			field.DataType = schema.DataType(val)
		}
	}

	if field.GORMDataType == "" {
		field.GORMDataType = field.DataType
	}

	if field.Size == 0 {
		switch f.Type.Kind {
		case spec.KindI64, spec.KindU64, spec.KindF64:
			field.Size = 64
		case spec.KindI8, spec.KindU8:
			field.Size = 8
		case spec.KindI16, spec.KindU16:
			field.Size = 16
		case spec.KindI32, spec.KindU32, spec.KindF32:
			field.Size = 32
		}
	}

	// setup permission
	if val, ok := field.TagSettings["-"]; ok {
		val = strings.ToLower(strings.TrimSpace(val))
		switch val {
		case "-":
			field.Creatable = false
			field.Updatable = false
			field.Readable = false
			field.DataType = ""
		case "all":
			field.Creatable = false
			field.Updatable = false
			field.Readable = false
			field.DataType = ""
			field.IgnoreMigration = true
		case "migration":
			field.IgnoreMigration = true
		}
	}

	if v, ok := field.TagSettings["->"]; ok {
		field.Creatable = false
		field.Updatable = false
		if strings.ToLower(v) == "false" {
			field.Readable = false
		} else {
			field.Readable = true
		}
	}

	if v, ok := field.TagSettings["<-"]; ok {
		field.Creatable = true
		field.Updatable = true

		if v != "<-" {
			if !strings.Contains(v, "create") {
				field.Creatable = false
			}

			if !strings.Contains(v, "update") {
				field.Updatable = false
			}
		}
	}

	// if _, ok := field.TagSettings["EMBEDDED"]; field.GORMDataType != schema.Time && field.GORMDataType != schema.Bytes &&
	// 	(ok || (fieldStruct.Anonymous && !isValuer && (field.Creatable || field.Updatable || field.Readable))) {
	// 	kind := reflect.Indirect(fieldValue).Kind()
	// 	switch kind {
	// 	case reflect.Struct:
	// 		var err error
	// 		field.Creatable = false
	// 		field.Updatable = false
	// 		field.Readable = false

	// 		cacheStore := &sync.Map{}
	// 		cacheStore.Store(embeddedCacheKey, true)
	// 		if field.EmbeddedSchema, err = getOrParse(fieldValue.Interface(), cacheStore, embeddedNamer{Table: schema.Table, Namer: schema.namer}); err != nil {
	// 			schema.err = err
	// 		}

	// 		for _, ef := range field.EmbeddedSchema.Fields {
	// 			ef.Schema = schema
	// 			ef.OwnerSchema = field.EmbeddedSchema
	// 			ef.BindNames = append([]string{fieldStruct.Name}, ef.BindNames...)
	// 			// index is negative means is pointer
	// 			if field.FieldType.Kind() == reflect.Struct {
	// 				ef.StructField.Index = append([]int{fieldStruct.Index[0]}, ef.StructField.Index...)
	// 			} else {
	// 				ef.StructField.Index = append([]int{-fieldStruct.Index[0] - 1}, ef.StructField.Index...)
	// 			}

	// 			if prefix, ok := field.TagSettings["EMBEDDEDPREFIX"]; ok && ef.DBName != "" {
	// 				ef.DBName = prefix + ef.DBName
	// 			}

	// 			if ef.PrimaryKey {
	// 				if val, ok := ef.TagSettings["PRIMARYKEY"]; ok && utils.CheckTruth(val) {
	// 					ef.PrimaryKey = true
	// 				} else if val, ok := ef.TagSettings["PRIMARY_KEY"]; ok && utils.CheckTruth(val) {
	// 					ef.PrimaryKey = true
	// 				} else {
	// 					ef.PrimaryKey = false

	// 					if val, ok := ef.TagSettings["AUTOINCREMENT"]; !ok || !utils.CheckTruth(val) {
	// 						ef.AutoIncrement = false
	// 					}

	// 					if ef.DefaultValue == "" {
	// 						ef.HasDefaultValue = false
	// 					}
	// 				}
	// 			}

	// 			for k, v := range field.TagSettings {
	// 				ef.TagSettings[k] = v
	// 			}
	// 		}
	// 	case reflect.Invalid, reflect.Uintptr, reflect.Array, reflect.Chan, reflect.Func, reflect.Interface,
	// 		reflect.Map, reflect.Ptr, reflect.Slice, reflect.UnsafePointer, reflect.Complex64, reflect.Complex128:
	// 		schema.err = fmt.Errorf("invalid embedded struct for %s's field %s, should be struct, but got %v", field.Schema.Name, field.Name, field.FieldType)
	// 	}
	// }

	return field, nil
}

// func (p *Processor) buildPolymorphicRelation(s *schema.Schema, relation *schema.Relationship, field *schema.Field, polymorphic string) error {
// 	relation.Polymorphic = &schema.Polymorphic{
// 		Value:           s.Table,
// 		PolymorphicType: relation.FieldSchema.FieldsByName[polymorphic+"Type"],
// 		PolymorphicID:   relation.FieldSchema.FieldsByName[polymorphic+"ID"],
// 	}

// 	if value, ok := field.TagSettings["POLYMORPHICVALUE"]; ok {
// 		relation.Polymorphic.Value = strings.TrimSpace(value)
// 	}

// 	if relation.Polymorphic.PolymorphicType == nil {
// 		return fmt.Errorf("invalid polymorphic type %v for %v on field %s, missing field %s", relation.FieldSchema, s, field.Name, polymorphic+"Type")
// 	}

// 	if relation.Polymorphic.PolymorphicID == nil {
// 		return fmt.Errorf("invalid polymorphic type %v for %v on field %s, missing field %s", relation.FieldSchema, s, field.Name, polymorphic+"ID")
// 	}

// 	relation.References = append(relation.References, &schema.Reference{
// 		PrimaryValue: relation.Polymorphic.Value,
// 		ForeignKey:   relation.Polymorphic.PolymorphicType,
// 	})

// 	primaryKeyField := s.PrioritizedPrimaryField
// 	if len(relation.foreignKeys) > 0 {
// 		if primaryKeyField = s.LookUpField(relation.foreignKeys[0]); primaryKeyField == nil || len(relation.foreignKeys) > 1 {
// 			return fmt.Errorf("invalid polymorphic foreign keys %+v for %v on field %s", relation.foreignKeys, s, field.Name)
// 		}
// 	}

// 	// use same data type for foreign keys
// 	if copyableDataType(primaryKeyField.DataType) {
// 		relation.Polymorphic.PolymorphicID.DataType = primaryKeyField.DataType
// 	}
// 	relation.Polymorphic.PolymorphicID.GORMDataType = primaryKeyField.GORMDataType
// 	if relation.Polymorphic.PolymorphicID.Size == 0 {
// 		relation.Polymorphic.PolymorphicID.Size = primaryKeyField.Size
// 	}

// 	relation.References = append(relation.References, &schema.Reference{
// 		PrimaryKey:    primaryKeyField,
// 		ForeignKey:    relation.Polymorphic.PolymorphicID,
// 		OwnPrimaryKey: true,
// 	})

// 	relation.Type = schema.RelationshipType("has")

// 	return nil
// }

// func (p *Processor) buildMany2ManyRelation(s *schema.Schema, relation *schema.Relationship, field *schema.Field, many2many string) error {
// 	relation.Type = schema.Many2Many

// 	var (
// 		err             error
// 		joinTableFields []reflect.StructField
// 		fieldsMap       = map[string]*schema.Field{}
// 		ownFieldsMap    = map[string]bool{} // fix self join many2many
// 		joinForeignKeys = toColumns(field.TagSettings["JOINFOREIGNKEY"])
// 		joinReferences  = toColumns(field.TagSettings["JOINREFERENCES"])
// 	)

// 	ownForeignFields := s.PrimaryFields
// 	refForeignFields := relation.FieldSchema.PrimaryFields

// 	if len(relation.foreignKeys) > 0 {
// 		ownForeignFields = []*schema.Field{}
// 		for _, foreignKey := range relation.foreignKeys {
// 			if field := s.LookUpField(foreignKey); field != nil {
// 				ownForeignFields = append(ownForeignFields, field)
// 			} else {
// 				return fmt.Errorf("invalid foreign key: %s", foreignKey)
// 			}
// 		}
// 	}

// 	if len(relation.primaryKeys) > 0 {
// 		refForeignFields = []*schema.Field{}
// 		for _, foreignKey := range relation.primaryKeys {
// 			if field := relation.FieldSchema.LookUpField(foreignKey); field != nil {
// 				refForeignFields = append(refForeignFields, field)
// 			} else {
// 				return fmt.Errorf("invalid foreign key: %s", foreignKey)
// 			}
// 		}
// 	}

// 	for idx, ownField := range ownForeignFields {
// 		joinFieldName := strings.Title(s.Name) + ownField.Name
// 		if len(joinForeignKeys) > idx {
// 			joinFieldName = strings.Title(joinForeignKeys[idx])
// 		}

// 		ownFieldsMap[joinFieldName] = true
// 		fieldsMap[joinFieldName] = ownField
// 		joinTableFields = append(joinTableFields, reflect.StructField{
// 			Name:    joinFieldName,
// 			PkgPath: ownField.StructField.PkgPath,
// 			Type:    ownField.StructField.Type,
// 			Tag:     removeSettingFromTag(ownField.StructField.Tag, "column", "autoincrement", "index", "unique", "uniqueindex"),
// 		})
// 	}

// 	for idx, relField := range refForeignFields {
// 		joinFieldName := strings.Title(relation.FieldSchema.Name) + relField.Name
// 		if len(joinReferences) > idx {
// 			joinFieldName = strings.Title(joinReferences[idx])
// 		}

// 		if _, ok := ownFieldsMap[joinFieldName]; ok {
// 			if field.Name != relation.FieldSchema.Name {
// 				joinFieldName = inflection.Singular(field.Name) + relField.Name
// 			} else {
// 				joinFieldName += "Reference"
// 			}
// 		}

// 		fieldsMap[joinFieldName] = relField
// 		joinTableFields = append(joinTableFields, reflect.StructField{
// 			Name:    joinFieldName,
// 			PkgPath: relField.StructField.PkgPath,
// 			Type:    relField.StructField.Type,
// 			Tag:     removeSettingFromTag(relField.StructField.Tag, "column", "autoincrement", "index", "unique", "uniqueindex"),
// 		})
// 	}

// 	joinTableFields = append(joinTableFields, reflect.StructField{
// 		Name: strings.Title(s.Name) + field.Name,
// 		Type: s.ModelType,
// 		Tag:  `gorm:"-"`,
// 	})

// 	if relation.JoinTable, err = schema.Parse(reflect.New(reflect.StructOf(joinTableFields)).Interface(), s.cacheStore, namer); err != nil {
// 		return err
// 	}
// 	relation.JoinTable.Name = many2many
// 	relation.JoinTable.Table = namer.JoinTableName(many2many)
// 	relation.JoinTable.PrimaryFields = make([]*schema.Field, 0, len(relation.JoinTable.Fields))

// 	relName := relation.Schema.Name
// 	relRefName := relation.FieldSchema.Name
// 	if relName == relRefName {
// 		relRefName = relation.Field.Name
// 	}

// 	if _, ok := relation.JoinTable.Relationships.Relations[relName]; !ok {
// 		relation.JoinTable.Relationships.Relations[relName] = &schema.Relationship{
// 			Name:        relName,
// 			Type:        schema.BelongsTo,
// 			Schema:      relation.JoinTable,
// 			FieldSchema: relation.Schema,
// 		}
// 	} else {
// 		relation.JoinTable.Relationships.Relations[relName].References = []*schema.Reference{}
// 	}

// 	if _, ok := relation.JoinTable.Relationships.Relations[relRefName]; !ok {
// 		relation.JoinTable.Relationships.Relations[relRefName] = &schema.Relationship{
// 			Name:        relRefName,
// 			Type:        schema.BelongsTo,
// 			Schema:      relation.JoinTable,
// 			FieldSchema: relation.FieldSchema,
// 		}
// 	} else {
// 		relation.JoinTable.Relationships.Relations[relRefName].References = []*schema.Reference{}
// 	}

// 	// build references
// 	for _, f := range relation.JoinTable.Fields {
// 		if f.Creatable || f.Readable || f.Updatable {
// 			// use same data type for foreign keys
// 			if copyableDataType(fieldsMap[f.Name].DataType) {
// 				f.DataType = fieldsMap[f.Name].DataType
// 			}
// 			f.GORMDataType = fieldsMap[f.Name].GORMDataType
// 			if f.Size == 0 {
// 				f.Size = fieldsMap[f.Name].Size
// 			}
// 			relation.JoinTable.PrimaryFields = append(relation.JoinTable.PrimaryFields, f)
// 			ownPrimaryField := s == fieldsMap[f.Name].Schema && ownFieldsMap[f.Name]

// 			if ownPrimaryField {
// 				joinRel := relation.JoinTable.Relationships.Relations[relName]
// 				joinRel.Field = relation.Field
// 				joinRel.References = append(joinRel.References, &schema.Reference{
// 					PrimaryKey: fieldsMap[f.Name],
// 					ForeignKey: f,
// 				})
// 			} else {
// 				joinRefRel := relation.JoinTable.Relationships.Relations[relRefName]
// 				if joinRefRel.Field == nil {
// 					joinRefRel.Field = relation.Field
// 				}
// 				joinRefRel.References = append(joinRefRel.References, &schema.Reference{
// 					PrimaryKey: fieldsMap[f.Name],
// 					ForeignKey: f,
// 				})
// 			}

// 			relation.References = append(relation.References, &schema.Reference{
// 				PrimaryKey:    fieldsMap[f.Name],
// 				ForeignKey:    f,
// 				OwnPrimaryKey: ownPrimaryField,
// 			})
// 		}
// 	}

// 	return nil
// }

// type guessLevel int

// const (
// 	guessGuess guessLevel = iota
// 	guessBelongs
// 	guessEmbeddedBelongs
// 	guessHas
// 	guessEmbeddedHas
// )

// func (p *Processor) guessRelation(s *schema.Schema, relation *schema.Relationship, field *schema.Field, cgl guessLevel) error {
// 	var (
// 		primaryFields, foreignFields []*schema.Field
// 		primarySchema, foreignSchema = s, relation.FieldSchema
// 		gl                           = cgl
// 	)

// 	if gl == guessGuess {
// 		if field.Schema == relation.FieldSchema {
// 			gl = guessBelongs
// 		} else {
// 			gl = guessHas
// 		}
// 	}

// 	reguessOrErr := func() {
// 		switch cgl {
// 		case guessGuess:
// 			guessRelation(s, relation, field, guessBelongs)
// 		case guessBelongs:
// 			guessRelation(s, relation, field, guessEmbeddedBelongs)
// 		case guessEmbeddedBelongs:
// 			guessRelation(s, relation, field, guessHas)
// 		case guessHas:
// 			guessRelation(s, relation, field, guessEmbeddedHas)
// 		// case guessEmbeddedHas:
// 		default:
// 			return fmt.Errorf("invalid field found for struct %v's field %s: define a valid foreign key for relations or implement the Valuer/Scanner interface", s, field.Name)
// 		}
// 	}

// 	switch gl {
// 	case guessBelongs:
// 		primarySchema, foreignSchema = relation.FieldSchema, s
// 	case guessEmbeddedBelongs:
// 		if field.OwnerSchema != nil {
// 			primarySchema, foreignSchema = relation.FieldSchema, field.OwnerSchema
// 		} else {
// 			reguessOrErr()
// 			return nil
// 		}
// 	case guessHas:
// 	case guessEmbeddedHas:
// 		if field.OwnerSchema != nil {
// 			primarySchema, foreignSchema = field.OwnerSchema, relation.FieldSchema
// 		} else {
// 			reguessOrErr()
// 			return nil
// 		}
// 	}

// 	if len(relation.foreignKeys) > 0 {
// 		for _, foreignKey := range relation.foreignKeys {
// 			if f := foreignSchema.LookUpField(foreignKey); f != nil {
// 				foreignFields = append(foreignFields, f)
// 			} else {
// 				reguessOrErr()
// 				return nil
// 			}
// 		}
// 	} else {
// 		var primaryFields []*schema.Field

// 		if len(relation.primaryKeys) > 0 {
// 			for _, primaryKey := range relation.primaryKeys {
// 				if f := primarySchema.LookUpField(primaryKey); f != nil {
// 					primaryFields = append(primaryFields, f)
// 				}
// 			}
// 		} else {
// 			primaryFields = primarySchema.PrimaryFields
// 		}

// 		for _, primaryField := range primaryFields {
// 			lookUpName := primarySchema.Name + primaryField.Name
// 			if gl == guessBelongs {
// 				lookUpName = field.Name + primaryField.Name
// 			}

// 			lookUpNames := []string{lookUpName}
// 			if len(primaryFields) == 1 {
// 				lookUpNames = append(lookUpNames, strings.TrimSuffix(lookUpName, primaryField.Name)+"ID", strings.TrimSuffix(lookUpName, primaryField.Name)+"Id", schema.namer.ColumnName(foreignSchema.Table, strings.TrimSuffix(lookUpName, primaryField.Name)+"ID"))
// 			}

// 			for _, name := range lookUpNames {
// 				if f := foreignSchema.LookUpField(name); f != nil {
// 					foreignFields = append(foreignFields, f)
// 					primaryFields = append(primaryFields, primaryField)
// 					break
// 				}
// 			}
// 		}
// 	}

// 	if len(foreignFields) == 0 {
// 		reguessOrErr()
// 		return nil
// 	} else if len(relation.primaryKeys) > 0 {
// 		for idx, primaryKey := range relation.primaryKeys {
// 			if f := primarySchema.LookUpField(primaryKey); f != nil {
// 				if len(primaryFields) < idx+1 {
// 					primaryFields = append(primaryFields, f)
// 				} else if f != primaryFields[idx] {
// 					reguessOrErr()
// 					return nil
// 				}
// 			} else {
// 				reguessOrErr()
// 				return nil
// 			}
// 		}
// 	} else if len(primaryFields) == 0 {
// 		if len(foreignFields) == 1 && primarySchema.PrioritizedPrimaryField != nil {
// 			primaryFields = append(primaryFields, primarySchema.PrioritizedPrimaryField)
// 		} else if len(primarySchema.PrimaryFields) == len(foreignFields) {
// 			primaryFields = append(primaryFields, primarySchema.PrimaryFields...)
// 		} else {
// 			reguessOrErr()
// 			return nil
// 		}
// 	}

// 	// build references
// 	for idx, foreignField := range foreignFields {
// 		// use same data type for foreign keys
// 		if copyableDataType(primaryFields[idx].DataType) {
// 			foreignField.DataType = primaryFields[idx].DataType
// 		}
// 		foreignField.GORMDataType = primaryFields[idx].GORMDataType
// 		if foreignField.Size == 0 {
// 			foreignField.Size = primaryFields[idx].Size
// 		}

// 		relation.References = append(relation.References, &schema.Reference{
// 			PrimaryKey:    primaryFields[idx],
// 			ForeignKey:    foreignField,
// 			OwnPrimaryKey: (s == primarySchema && gl == guessHas) || (field.OwnerSchema == primarySchema && gl == guessEmbeddedHas),
// 		})
// 	}

// 	if gl == guessHas || gl == guessEmbeddedHas {
// 		relation.Type = schema.RelationshipType("has")
// 	} else {
// 		relation.Type = schema.BelongsTo
// 	}

// 	return nil
// }
