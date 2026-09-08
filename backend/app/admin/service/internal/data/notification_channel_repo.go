package data

import (
	"context"
	"time"

	"entgo.io/ent/dialect/sql"
	bLogger "github.com/tx7do/kratos-bootstrap/logger"
	"github.com/tx7do/kratos-bootstrap/bootstrap"
	paginationV1 "github.com/tx7do/go-crud/api/gen/go/pagination/v1"
	entCrud "github.com/tx7do/go-crud/entgo"
	"github.com/tx7do/go-utils/copierutil"
	"github.com/tx7do/go-utils/mapper"
	"github.com/tx7do/go-utils/trans"

	"go-wind-admin/app/admin/service/internal/data/ent"
	"go-wind-admin/app/admin/service/internal/data/ent/notificationchannel"
	"go-wind-admin/app/admin/service/internal/data/ent/predicate"
	"go-wind-admin/pkg/crypto"

	adminV1 "go-wind-admin/api/gen/go/admin/service/v1"
	notificationChannelV1 "go-wind-admin/api/gen/go/notification_channel/service/v1"
)

type NotificationChannelRepo struct {
	entClient *entCrud.EntClient[*ent.Client]
	log       *bLogger.Helper

	mapper        *mapper.CopierMapper[notificationChannelV1.NotificationChannel, ent.NotificationChannel]
	typeConverter *mapper.EnumTypeConverter[notificationChannelV1.NotificationChannel_Type, notificationchannel.Type]
	tlsConverter  *mapper.EnumTypeConverter[notificationChannelV1.NotificationChannel_TlsMode, notificationchannel.SMTPTLS]

	repository *entCrud.Repository[
		ent.NotificationChannelQuery, ent.NotificationChannelSelect,
		ent.NotificationChannelCreate, ent.NotificationChannelCreateBulk,
		ent.NotificationChannelUpdate, ent.NotificationChannelUpdateOne,
		ent.NotificationChannelDelete,
		predicate.NotificationChannel,
		notificationChannelV1.NotificationChannel, ent.NotificationChannel,
	]
}

func NewNotificationChannelRepo(ctx *bootstrap.Context, entClient *entCrud.EntClient[*ent.Client]) *NotificationChannelRepo {
	repo := &NotificationChannelRepo{
		log:       ctx.NewLoggerHelper("notification-channel/repo/admin-service"),
		entClient: entClient,
		mapper:    mapper.NewCopierMapper[notificationChannelV1.NotificationChannel, ent.NotificationChannel](),
		typeConverter: mapper.NewEnumTypeConverter[notificationChannelV1.NotificationChannel_Type, notificationchannel.Type](
			notificationChannelV1.NotificationChannel_Type_name, notificationChannelV1.NotificationChannel_Type_value,
		),
		tlsConverter: mapper.NewEnumTypeConverter[notificationChannelV1.NotificationChannel_TlsMode, notificationchannel.SMTPTLS](
			notificationChannelV1.NotificationChannel_TlsMode_name, notificationChannelV1.NotificationChannel_TlsMode_value,
		),
	}

	repo.init()

	return repo
}

func (r *NotificationChannelRepo) init() {
	r.repository = entCrud.NewRepository[
		ent.NotificationChannelQuery, ent.NotificationChannelSelect,
		ent.NotificationChannelCreate, ent.NotificationChannelCreateBulk,
		ent.NotificationChannelUpdate, ent.NotificationChannelUpdateOne,
		ent.NotificationChannelDelete,
		predicate.NotificationChannel,
		notificationChannelV1.NotificationChannel, ent.NotificationChannel,
	](r.mapper)

	r.mapper.AppendConverters(copierutil.NewTimeStringConverterPair())
	r.mapper.AppendConverters(copierutil.NewTimeTimestamppbConverterPair())

	r.mapper.AppendConverters(r.typeConverter.NewConverterPair())
	r.mapper.AppendConverters(r.tlsConverter.NewConverterPair())
}

// List 分页查询通知渠道（密码字段不出现在 DTO，靠 HasPassword 标识）。
func (r *NotificationChannelRepo) List(ctx context.Context, req *paginationV1.PagingRequest) (*notificationChannelV1.ListNotificationChannelResponse, error) {
	if req == nil {
		return nil, adminV1.ErrorBadRequest("invalid parameter")
	}

	builder := r.entClient.Client().NotificationChannel.Query()

	ret, err := r.repository.ListWithPaging(ctx, builder, builder.Clone(), req)
	if err != nil {
		return nil, err
	}
	if ret == nil {
		return &notificationChannelV1.ListNotificationChannelResponse{Total: 0, Items: nil}, nil
	}

	// 填充 HasPassword 标识：按批量查询密码字段有无
	r.queryHasPasswordByIDs(ctx, ret.Items)

	return &notificationChannelV1.ListNotificationChannelResponse{
		Total: ret.Total,
		Items: ret.Items,
	}, nil
}

// queryHasPasswordByIDs 为 DTO 列表填充 HasPassword 标识。
func (r *NotificationChannelRepo) queryHasPasswordByIDs(ctx context.Context, items []*notificationChannelV1.NotificationChannel) {
	if len(items) == 0 {
		return
	}
	entities, err := r.entClient.Client().NotificationChannel.Query().
		Select(notificationchannel.FieldID, notificationchannel.FieldSMTPPassword).
		All(ctx)
	if err != nil {
		r.log.Errorf(ctx, "query password flags failed: %s", err.Error())
		return
	}
	hasPwd := make(map[uint32]bool, len(entities))
	for _, e := range entities {
		hasPwd[e.ID] = e.SMTPPassword != nil
	}
	for _, it := range items {
		it.HasPassword = trans.Ptr(hasPwd[it.GetId()])
	}
}

func (r *NotificationChannelRepo) IsExist(ctx context.Context, id uint32) (bool, error) {
	exist, err := r.entClient.Client().NotificationChannel.Query().
		Where(notificationchannel.IDEQ(id)).
		Exist(ctx)
	if err != nil {
		r.log.Errorf(ctx, "query notification channel exist failed: %s", err.Error())
		return false, adminV1.ErrorInternalServerError("query exist failed")
	}
	return exist, nil
}

// Get 按 ID 查询渠道（脱敏：DTO 不含密码，仅 HasPassword 标识）。
func (r *NotificationChannelRepo) Get(ctx context.Context, id uint32) (*notificationChannelV1.NotificationChannel, error) {
	entity, err := r.entClient.Client().NotificationChannel.Get(ctx, id)
	if err != nil {
		r.log.Errorf(ctx, "get notification channel [%d] failed: %s", id, err.Error())
		return nil, adminV1.ErrorNotFound("notification channel not found")
	}
	dto := r.mapper.ToDTO(entity)
	dto.HasPassword = trans.Ptr(entity.SMTPPassword != nil)
	return dto, nil
}

// Create 创建渠道；password 明文经 EncryptIfNeeded 加密后落库。
func (r *NotificationChannelRepo) Create(ctx context.Context, req *notificationChannelV1.CreateNotificationChannelRequest, operatorID uint32) (uint32, error) {
	if req == nil || req.Data == nil {
		return 0, adminV1.ErrorBadRequest("invalid request")
	}
	if req.Data.GetName() == "" {
		return 0, adminV1.ErrorBadRequest("channel name is required")
	}

	encrypted, err := crypto.EncryptIfNeeded(req.GetPassword())
	if err != nil {
		r.log.Errorf(ctx, "encrypt smtp password failed: %s", err.Error())
		return 0, adminV1.ErrorInternalServerError("encrypt password failed")
	}

	builder := r.entClient.Client().NotificationChannel.Create().
		SetName(req.Data.GetName()).
		SetNillableType(r.typeConverter.ToEntity(req.Data.Type)).
		SetNillableSMTPHost(req.Data.SmtpHost).
		SetNillableSMTPPort(req.Data.SmtpPort).
		SetNillableSMTPUsername(req.Data.SmtpUsername).
		SetNillableSMTPFrom(req.Data.SmtpFrom).
		SetNillableSMTPTLS(r.tlsConverter.ToEntity(req.Data.SmtpTls)).
		SetStatus(statusFromEnabled(req.Data.GetEnabled())).
		SetNillableRemark(req.Data.Remark).
		SetCreatedBy(operatorID).
		SetCreatedAt(time.Now())

	if req.GetPassword() != "" {
		builder.SetSMTPPassword(encrypted)
	}

	created, err := builder.Save(ctx)
	if err != nil {
		r.log.Errorf(ctx, "insert notification channel failed: %s", err.Error())
		return 0, adminV1.ErrorInternalServerError("insert notification channel failed")
	}

	return created.ID, nil
}

// Update 更新渠道；password 留空表示不修改已存密码。
func (r *NotificationChannelRepo) Update(ctx context.Context, req *notificationChannelV1.UpdateNotificationChannelRequest, operatorID uint32) error {
	if req == nil || req.Data == nil {
		return adminV1.ErrorBadRequest("invalid request")
	}
	if req.GetId() == 0 {
		return adminV1.ErrorBadRequest("id is required")
	}

	builder := r.entClient.Client().NotificationChannel.Update()
	err := r.repository.UpdateX(ctx, builder, req.Data, req.GetUpdateMask(),
		func(dto *notificationChannelV1.NotificationChannel) {
			builder.
				SetNillableName(req.Data.Name).
				SetNillableSMTPHost(req.Data.SmtpHost).
				SetNillableSMTPPort(req.Data.SmtpPort).
				SetNillableSMTPUsername(req.Data.SmtpUsername).
				SetNillableSMTPFrom(req.Data.SmtpFrom).
				SetNillableSMTPTLS(r.tlsConverter.ToEntity(req.Data.SmtpTls)).
				SetNillableStatus(r.statusFromProto(req.Data.Enabled)).
				SetNillableRemark(req.Data.Remark).
				SetNillableUpdatedBy(trans.Ptr(operatorID)).
				SetUpdatedAt(time.Now())
			if req.GetPassword() != "" {
				encrypted, encErr := crypto.EncryptIfNeeded(req.GetPassword())
				if encErr != nil {
					r.log.Errorf(ctx, "encrypt smtp password failed: %s", encErr.Error())
					return
				}
				builder.SetSMTPPassword(encrypted)
			}
		},
		func(s *sql.Selector) {
			s.Where(sql.EQ(notificationchannel.FieldID, req.GetId()))
		},
	)

	return err
}

func (r *NotificationChannelRepo) Delete(ctx context.Context, id uint32) error {
	if id == 0 {
		return adminV1.ErrorBadRequest("id is required")
	}

	if err := r.entClient.Client().NotificationChannel.DeleteOneID(id).Exec(ctx); err != nil {
		r.log.Errorf(ctx, "delete notification channel [%d] failed: %s", id, err.Error())
		return adminV1.ErrorInternalServerError("delete notification channel failed")
	}

	return nil
}

// SmtpAccount 发送邮件所需的解密后 SMTP 配置（仅服务层内部使用，禁止外传）。
type SmtpAccount struct {
	Host     string
	Port     uint32
	Username string
	Password string
	From     string
	TlsMode  string
	Enabled  bool
}

// GetFirstEnabledEmailChannel 取第一个启用的 EMAIL 渠道（找回密码/验证码发送用）。
func (r *NotificationChannelRepo) GetFirstEnabledEmailChannel(ctx context.Context) (*SmtpAccount, error) {
	entities, err := r.entClient.Client().NotificationChannel.Query().
		Where(
			notificationchannel.TypeEQ(notificationchannel.TypeEmail),
			notificationchannel.StatusEQ(notificationchannel.StatusOn),
		).
		Order(ent.Asc(notificationchannel.FieldID)).
		Limit(1).
		All(ctx)
	if err != nil {
		r.log.Errorf(ctx, "query enabled email channel failed: %s", err.Error())
		return nil, adminV1.ErrorInternalServerError("query email channel failed")
	}
	if len(entities) == 0 {
		return nil, adminV1.ErrorNotFound("no enabled email channel")
	}
	e := entities[0]

	password := ""
	if e.SMTPPassword != nil {
		decrypted, decErr := crypto.DecryptIfNeeded(*e.SMTPPassword)
		if decErr != nil {
			r.log.Errorf(ctx, "decrypt smtp password failed for channel [%d]: %s", e.ID, decErr.Error())
			return nil, adminV1.ErrorInternalServerError("decrypt password failed")
		}
		password = decrypted
	}

	return &SmtpAccount{
		Host:     derefStr(e.SMTPHost),
		Port:     derefUint32(e.SMTPPort),
		Username: derefStr(e.SMTPUsername),
		Password: password,
		From:     derefStr(e.SMTPFrom),
		TlsMode:  string(*e.SMTPTLS),
		Enabled:  true,
	}, nil
}

// GetDecryptedSmtpAccount 取渠道的解密 SMTP 配置。
func (r *NotificationChannelRepo) GetDecryptedSmtpAccount(ctx context.Context, id uint32) (*SmtpAccount, error) {
	entity, err := r.entClient.Client().NotificationChannel.Query().
		Where(notificationchannel.IDEQ(id)).
		Only(ctx)
	if err != nil {
		r.log.Errorf(ctx, "get notification channel [%d] failed: %s", id, err.Error())
		return nil, adminV1.ErrorNotFound("notification channel not found")
	}

	password := ""
	if entity.SMTPPassword != nil {
		decrypted, decErr := crypto.DecryptIfNeeded(*entity.SMTPPassword)
		if decErr != nil {
			r.log.Errorf(ctx, "decrypt smtp password failed for channel [%d]: %s", id, decErr.Error())
			return nil, adminV1.ErrorInternalServerError("decrypt password failed")
		}
		password = decrypted
	}

	return &SmtpAccount{
		Host:     derefStr(entity.SMTPHost),
		Port:     derefUint32(entity.SMTPPort),
		Username: derefStr(entity.SMTPUsername),
		Password: password,
		From:     derefStr(entity.SMTPFrom),
		TlsMode:  derefStrP(entity.SMTPTLS),
		Enabled:  entity.Status != nil && *entity.Status == notificationchannel.StatusOn,
	}, nil
}

// statusFromEnabled DTO enabled 布尔 → ent status 枚举
func statusFromEnabled(enabled bool) notificationchannel.Status {
	if enabled {
		return notificationchannel.StatusOn
	}
	return notificationchannel.StatusOff
}

// statusFromProto DTO optional enabled → ent status 枚举指针（nil 表示不修改）
func (r *NotificationChannelRepo) statusFromProto(enabled *bool) *notificationchannel.Status {
	if enabled == nil {
		return nil
	}
	s := statusFromEnabled(*enabled)
	return &s
}
